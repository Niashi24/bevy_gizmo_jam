use crate::tileset::grid::Grid;
use crate::tileset::tile::{RampOrientation, Tile, TileImageUnknownPixel};
use avian2d::prelude::*;
use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext};
use bevy::prelude::*;
use geo::{BooleanOps, Centroid, CoordsIter, MultiPolygon, Scale, Translate, TriangulateEarcut};
use std::collections::VecDeque;
use std::io;
use itertools::Itertools;
use thiserror::Error;

#[test]
fn load_tilemap() -> io::Result<()> {
    let grid: Grid<Tile> = (&image::io::Reader::open("assets/levels/test_level.png")
        .unwrap()
        .decode()
        .unwrap())
        .try_into()
        .unwrap();

    println!("{}", grid);

    Ok(())
}

#[derive(Asset, Debug, Reflect, Clone)]
pub struct TileGridAsset {
    pub grid: Grid<Tile>,
    pub collider: Vec<([Vec2; 3], Vec2)>,
}

impl TileGridAsset {
    pub fn new(grid: Grid<Tile>, tile_size: f32) -> Self {
        let polygons = grid
            .iter()
            .flat_map(|((x, y), t)| {
                t.to_collider_verts().map(|mut p| {
                    p.translate_mut(
                        x as f32 * tile_size,
                        y as f32 * -tile_size,
                    );
                    p.scale_xy_mut(tile_size, tile_size);

                    p
                })
            })
            .map(|p| MultiPolygon::new(vec![p]))
            .collect::<Vec<_>>();

        let polygons =
            divide_reduce(polygons, |a, b| a.union(&b))
                .unwrap_or(MultiPolygon::new(vec![]));
        
        let tris = polygons.into_iter()
            .flat_map(|x| x.earcut_triangles_iter())
            .map(|tri| {
                let (c_x, c_y) = tri.centroid().0.x_y();
                let d = Vec2::new(c_x, c_y);
                let tri = tri.to_array()
                    .map(|p| Vec2::new(p.x, p.y))
                    .map(|p| p - d);

                (tri, d)
            })
            .collect_vec();
        
        Self {
            grid,
            collider: tris,
        }
    }
}

#[derive(Default)]
pub struct TileGridAssetLoader;

/// Possible errors from loading a TileGridAsset
#[derive(Debug, Error)]
pub enum TileGridAssetLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] io::Error),
    #[error("Could not load image: {0}")]
    Image(#[from] image::ImageError),
    #[error("Could not parse image: {0}")]
    Pixel(#[from] TileImageUnknownPixel),
}

impl AssetLoader for TileGridAssetLoader {
    type Asset = TileGridAsset;
    type Settings = ();
    type Error = TileGridAssetLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let img = image::load_from_memory(bytes.as_slice())?;

        let grid = (&img).try_into()?;
        
        // println!("{}", grid);
        Ok(TileGridAsset::new(grid, 16.0))
    }
}


#[derive(Component, Default, Clone)]
pub struct TileGridSettings {
    pub solid_texture: Handle<Image>,
    pub ramp_texture: Handle<Image>,
    pub tile_size: f32,
}

#[derive(Bundle, Default)]
pub struct TileGridBundle {
    /// The settings for this TileGrid
    pub settings: TileGridSettings,
    /// The tile grid being spawned
    pub tile_grid: Handle<TileGridAsset>,
    /// The local transform of the sprite, relative to its parent.
    pub transform: Transform,
    /// The absolute transform of the sprite. This should generally not be written to directly.
    pub global_transform: GlobalTransform,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
    /// Inherited visibility of an entity.
    pub inherited_visibility: InheritedVisibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub view_visibility: ViewVisibility,
    /// A marker telling that the asset is currently being loaded
    pub loading_marker: TileGridLoadingMarker,
}

#[derive(Component, Default)]
#[component(storage = "SparseSet")]
pub struct TileGridLoadingMarker;

#[derive(Event)]
pub struct TileGridLoadEvent(pub TileGridAsset, pub TileGridSettings, pub Entity);

pub(crate) fn spawn_grid(
    mut commands: Commands,
    query: Query<(Entity, &TileGridSettings, &Handle<TileGridAsset>), With<TileGridLoadingMarker>>,
    grid_assets: Res<Assets<TileGridAsset>>,
    mut load_event: EventWriter<TileGridLoadEvent>,
) {
    for (entity, settings, tile_grid) in query.iter() {
        let Some(grid) = grid_assets.get(tile_grid) else {
            continue;
        };

        load_event.send(TileGridLoadEvent(grid.clone(), settings.clone(), entity));

        // info!("loaded tilemap!");

        let mut commands = commands.entity(entity);

        commands.remove::<TileGridLoadingMarker>();
    }
}

pub fn spawn_ramps(mut commands: Commands, mut tile_grid: EventReader<TileGridLoadEvent>, mut asset_server: ResMut<AssetServer>) {
    for TileGridLoadEvent(grid, settings, parent) in tile_grid.read() {
        let colliders_parent = commands
            .spawn_empty()
            .insert(Name::new("Background Tiles"))
            .insert(SpatialBundle::default())
            .id();
        commands.entity(*parent).add_child(colliders_parent);
        let mut commands = commands.entity(colliders_parent);

        for ((x, y), tile) in grid.grid.iter() {
            let x = x as f32 * settings.tile_size;
            let y = y as f32 * settings.tile_size;

            let transform = Transform::from_xyz(x, -y, 0.0);

            commands.with_children(|parent| match tile {
                Tile::Solid => {
                    parent.spawn((
                        Name::new("Solid"),
                        SpriteBundle {
                            transform,
                            texture: settings.solid_texture.clone_weak(),
                            ..default()
                        },
                    ));
                }
                Tile::Ramp(orientation) => {
                    let (flip_x, flip_y) = match orientation {
                        RampOrientation::SW => (false, false),
                        RampOrientation::SE => (true, false),
                        RampOrientation::NE => (true, true),
                        RampOrientation::NW => (false, true),
                    };

                    parent.spawn((
                        Name::new(format!("Ramp: {:?}", orientation)),
                        SpriteBundle {
                            transform,
                            texture: settings.ramp_texture.clone_weak(),
                            sprite: Sprite {
                                flip_x,
                                flip_y,
                                ..default()
                            },
                            ..default()
                        },
                    ));
                }
                Tile::Goal => {
                    parent.spawn((
                        Name::new("Goal"),
                        SpriteBundle {
                            transform,
                            texture: asset_server.load("textures/goal.png"),
                            ..default()
                        },
                        Goal,
                        Collider::rectangle(settings.tile_size, settings.tile_size),
                        Sensor,
                    ));
                }
                _ => {}
            });
        }
    }
}

#[derive(Component)]
struct Goal;

fn divide_reduce<T>(list: Vec<T>, mut reduction: impl FnMut(T, T) -> T) -> Option<T> {
    let mut queue = VecDeque::from(list);

    while queue.len() > 1 {
        for _ in 0..(queue.len() / 2) {
            let (one, two) = (queue.pop_front().unwrap(), queue.pop_front().unwrap());
            queue.push_back(reduction(one, two));
        }
    }

    queue.pop_back()
}

pub fn spawn_background_tiles(
    mut commands: Commands,
    mut tile_grid: EventReader<TileGridLoadEvent>,
) {
    for TileGridLoadEvent(grid, settings, parent) in tile_grid.read() {

        commands.entity(*parent).with_children(|parent| {
            for ([a, b, c], centroid) in grid.collider.iter().copied() {
                // Spawn triangle
                parent.spawn((
                    Name::new("Triangle"),
                    Collider::triangle(a, b, c),
                    SpatialBundle::from_transform(Transform::from_translation(centroid.extend(0.0))),
                    RigidBody::Static,
                ));
            }
        });
    }
}
