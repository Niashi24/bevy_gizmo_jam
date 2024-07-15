use std::fmt::{Display, Formatter};
use bevy::prelude::Reflect;
use image::{DynamicImage, GenericImageView, Pixel, Rgba};
use hex_literal::hex;
use itertools::Itertools;
use thiserror::Error;
use crate::tileset::grid::Grid;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Reflect)]
pub enum Tile {
    Solid,
    Air,
    Ramp(RampOrientation)
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Reflect)]
pub enum RampOrientation {
    SW,
    SE,
    NE,
    NW,
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
        const SOLID: [u8; 4] = hex!("000000ff");
        
        match value {
            Rgba(NW) => Ok(Tile::Ramp(RampOrientation::NW)),
            Rgba(NE) => Ok(Tile::Ramp(RampOrientation::NE)),
            Rgba(SE) => Ok(Tile::Ramp(RampOrientation::SE)),
            Rgba(SW) => Ok(Tile::Ramp(RampOrientation::SW)),
            Rgba(SOLID) => Ok(Tile::Solid),
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
#[error("Unknown pixel {0:?} at ({x},{y})")]
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
        (0..value.height()).map(move |y| {
            (0..value.width()).map(move |x| {
                Tile::try_from(value.get_pixel(x, y))
                    .map_err(|p| TileImageUnknownPixel::new(x, y, p.0))
            }).collect::<Result<Vec<_>, _>>()
        })
            .collect::<Result<Vec<_>, _>>()
            .map(Grid::new)
    }
}
