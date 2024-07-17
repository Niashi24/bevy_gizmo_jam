use std::fmt::{Display, Formatter};
use bevy::prelude::{Reflect, Vec2};
use geo::{LineString, Polygon};
use image::{DynamicImage, GenericImageView, Rgba};
use hex_literal::hex;
use thiserror::Error;
use crate::tileset::grid::Grid;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Reflect)]
pub enum Tile {
    Solid,
    Air,
    Player,
    Ramp(RampOrientation),
}

impl Tile {
    pub fn to_collider_verts(&self) -> Option<Polygon<f32>> {
        match self {
            Tile::Solid => Some(Polygon::new(LineString::from(vec![(-0.5, -0.5), (-0.5, 0.5), (0.5, 0.5), (0.5, -0.5), (-0.5, -0.5)]), vec![])),
            Tile::Ramp(x) => {
                let mut tri: Vec<_> = x.to_triangle().map(|x| (x.x, x.y)).into();
                tri.push(tri[0]);
                Some(Polygon::new(LineString::from(tri), vec![]))
            },
            Tile::Air => None,
            Tile::Player => None,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Reflect)]
pub enum RampOrientation {
    SW,
    SE,
    NE,
    NW,
}

impl RampOrientation {
    pub fn to_triangle(&self) -> [Vec2; 3] {
        match self {
            RampOrientation::SW => [Vec2::ZERO, Vec2::Y, Vec2::X],
            RampOrientation::SE => [Vec2::X, Vec2::ZERO, Vec2::ONE],
            RampOrientation::NE => [Vec2::ONE, Vec2::X, Vec2::Y],
            RampOrientation::NW => [Vec2::Y, Vec2::ONE, Vec2::ZERO],
        }
            .map(|x| x - Vec2::splat(0.5))
    }
}

#[derive(Error, Debug)]
#[error("Unknown Pixel: {0:?}")]
pub struct UnknownPixel(pub Rgba<u8>);

impl TryFrom<Rgba<u8>> for Tile {
    type Error = UnknownPixel;

    fn try_from(value: Rgba<u8>) -> Result<Self, Self::Error> {
        const NW: [u8; 4] = hex!("ee4080ff");
        const NE: [u8; 4] = hex!("124080ff");
        const SE: [u8; 4] = hex!("28b3cbff");
        const SW: [u8; 4] = hex!("eebf80ff");
        const PLAYER: [u8; 4] = [255, 0, 0, 255];
        const SOLID: [u8; 4] = hex!("000000ff");

        match value {
            Rgba(NW) => Ok(Tile::Ramp(RampOrientation::NW)),
            Rgba(NE) => Ok(Tile::Ramp(RampOrientation::NE)),
            Rgba(SE) => Ok(Tile::Ramp(RampOrientation::SE)),
            Rgba(SW) => Ok(Tile::Ramp(RampOrientation::SW)),
            Rgba(SOLID) => Ok(Tile::Solid),
            Rgba(PLAYER) => Ok(Tile::Player),
            Rgba([_, _, _, 0]) => Ok(Tile::Air),
            _ => Err(UnknownPixel(value)),
        }
    }
}

impl TryFrom<char> for Tile {
    type Error = char;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'X' => Ok(Self::Solid),
            ' ' => Ok(Self::Air),
            '\\' => Ok(Self::Ramp(RampOrientation::SW)),
            '/' => Ok(Self::Ramp(RampOrientation::SE)),
            '`' => Ok(Self::Ramp(RampOrientation::NE)),
            ',' => Ok(Self::Ramp(RampOrientation::NW)),
            c => Err(c),
        }
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match *self {
            Tile::Solid => '■'.to_string(),
            Tile::Air => ' '.to_string(),
            Tile::Ramp(x) => x.to_string(),
            Tile::Player => '🏃'.to_string(),
        })
    }
}

impl Display for RampOrientation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match *self {
            RampOrientation::SW => '◣',
            RampOrientation::SE => '◢',
            RampOrientation::NE => '◥',
            RampOrientation::NW => '◤',
        })
    }
}

#[derive(Debug, Clone, Error)]
#[error("Unknown pixel {pixel:?} at ({x},{y})")]
pub struct TileImageUnknownPixel {
    pub x: u32,
    pub y: u32,
    pub pixel: Rgba<u8>,
}

impl TileImageUnknownPixel {
    pub fn new(x: u32, y: u32, pixel: Rgba<u8>) -> Self {
        Self {
            x,
            y,
            pixel,
        }
    }
}

impl TryFrom<&DynamicImage> for Grid<Tile> {
    type Error = TileImageUnknownPixel;

    fn try_from(value: &DynamicImage) -> Result<Self, Self::Error> {
        Grid::try_from_iter((0..value.height()).map(move |y| {
            (0..value.width()).map(move |x| {
                Tile::try_from(value.get_pixel(x, y))
            })
        }))
            .map_err(|(x, y, e)| TileImageUnknownPixel::new(x as u32, y as u32, e.0))
    }
}
