use std::io;
use avian2d::prelude::*;
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext};
use bevy::asset::io::Reader;
use bevy::prelude::*;
use geo::{BooleanOps, CoordsIter, MultiPolygon, Translate};
use thiserror::Error;
use crate::tileset::grid::Grid;
use crate::tileset::tile::{RampOrientation, Tile, TileImageUnknownPixel};

#[test]
fn load_tilemap() -> io::Result<()> {
    let grid: Grid<Tile> = (&image::io::Reader::open("assets/levels/test_level.png").unwrap()
        .decode().unwrap())
        .try_into().unwrap();

    println!("{}", grid);

    Ok(())
}

#[derive(Asset, Debug, Reflect, Clone)]
pub struct TileGridAsset(pub Grid<Tile>);

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
        Ok(TileGridAsset(grid))
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
    query: Query<
        (
            Entity,
            &TileGridSettings,
            &Handle<TileGridAsset>
        ),
        With<TileGridLoadingMarker>
    >,
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

pub fn spawn_ramps(
    mut commands: Commands,
    mut tile_grid: EventReader<TileGridLoadEvent>,
) {
    for TileGridLoadEvent(grid, settings, parent) in tile_grid.read() {
        commands.entity(*parent).with_children(|parent| {
            for ((x, y), orientation) in grid.0.iter().flat_map(|(p, t)| match t {
                Tile::Ramp(o) => Some((p, o)),
                _ => None,
            }) {
                let x = x as f32 * settings.tile_size;
                let y = y as f32 * settings.tile_size;

                let transform = Transform::from_xyz(x, -y, 0.0);

                let (flip_x, flip_y) = match orientation {
                    RampOrientation::SW => (false, false),
                    RampOrientation::SE => (true, false),
                    RampOrientation::NE => (true, true),
                    RampOrientation::NW => (false, true),
                };

                let [p1, p2, p3] = orientation.to_triangle()
                    .map(|v| v * settings.tile_size);

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
                    Collider::triangle(p1, p2, p3),
                    RigidBody::Static,
                ));
            }
        });
    }
}

pub fn spawn_background_tiles(
    mut commands: Commands,
    mut tile_grid: EventReader<TileGridLoadEvent>,
) {
    for TileGridLoadEvent(grid, settings, parent) in tile_grid.read() {

        let now = std::time::Instant::now();

        let polygons = grid.0.iter()
            .flat_map(|((x, y), t)| t.to_collider_verts()
                .map(|mut p| {
                    p.translate_mut(x as f32 * settings.tile_size, y as f32 * -settings.tile_size);

                    p
                }))
            .fold(MultiPolygon::new(vec![]), |acc, p| acc.union(&p.into()));

        info!("Calculated polygons in {:?}", now.elapsed());

        dbg!(polygons.0.len());
        dbg!(polygons.0.iter().map(|p| p.interiors().len()).sum::<usize>());

        // dbg!(&polygons);

        let colliders_parent = commands.spawn_empty().id();
        commands.entity(*parent).add_child(colliders_parent);
        let mut commands = commands.entity(colliders_parent);

        commands.with_children(|parent| {
            for p in polygons.0 {
                let vertices = p.exterior_coords_iter()
                    .map(|p| Vec2::new(p.x, p.y))
                    .collect::<Vec<_>>();

                parent.spawn((
                    Collider::polyline(vertices, None),
                    RigidBody::Static,
                ));
            }
        });

        // for polygon in polygons {
            
        // }

        // commands.entity(*parent).with_children(|parent| {
        //     for r in rectangles {
        //         let start_x = r.start_x as f32 * settings.tile_size;
        //         let start_y = r.start_y as f32 * settings.tile_size;
        //         let width = (r.end_x - r.start_x + 1) as f32 * settings.tile_size;
        //         let height = (r.end_y - r.start_y + 1) as f32 * settings.tile_size;
        //
        //         let x = start_x + (width / 2.0);
        //         let y = start_y + (height / 2.0);
        //
        //         // Dummy implementation of physics body and shape creation
        //         let body = x; // Replace with actual body creation
        //         let shape = (x, y, width, height); // Replace with actual shape creation
        //
        //         parent.spawn((
        //             Name::new("Solid"),
        //             SpriteBundle {
        //                 transform: Transform::from_xyz(x, -y, 0.0),
        //                 texture: settings.solid_texture.clone_weak(),
        //                 ..default()
        //             },
        //             Collider::rectangle(width, height),
        //             RigidBody::Static,
        //         ));
        //     }
        // });


        // for ((x, y), tile) in grid.0.iter() {
        //     let x = x as f32 * settings.tile_size;
        //     let y = y as f32 * settings.tile_size;
        //     
        //     let transform = Transform::from_xyz(x, -y, 0.0);
        // 
        //     commands.entity(*parent).with_children(|parent| {
        //         match tile {
        //             Tile::Solid => {
        //                 parent.spawn((
        //                     Name::new("Solid"),
        //                     SpriteBundle {
        //                         transform,
        //                         texture: settings.solid_texture.clone_weak(),
        //                         ..default()
        //                     },
        //                     Collider::rectangle(settings.tile_size, settings.tile_size),
        //                     RigidBody::Static,
        //                 ));
        //             }
        //             Tile::Ramp(orientation) => {
        //                 let (flip_x, flip_y) = match orientation {
        //                     RampOrientation::SW => (false, false),
        //                     RampOrientation::SE => (true, false),
        //                     RampOrientation::NE => (true, true),
        //                     RampOrientation::NW => (false, true),
        //                 };
        // 
        //                 let [p1, p2, p3] = orientation.to_triangle()
        //                     .map(|v| v * settings.tile_size);
        //                 
        //                 parent.spawn((
        //                     Name::new(format!("Ramp: {:?}", orientation)),
        //                     SpriteBundle {
        //                         transform,
        //                         texture: settings.ramp_texture.clone_weak(),
        //                         sprite: Sprite {
        //                             flip_x,
        //                             flip_y,
        //                             ..default()
        //                         },
        //                         ..default()
        //                     },
        //                     Collider::triangle(p1, p2, p3),
        //                     RigidBody::Static,
        //                 ));
        //             }
        //             _ => {}
        //         }
        //     });
        // }
    }
}