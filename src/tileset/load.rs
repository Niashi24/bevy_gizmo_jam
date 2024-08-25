use crate::tileset::grid::Grid;
use crate::tileset::tile::{RampOrientation, Tile, TileImageUnknownPixel};
use avian2d::prelude::*;
use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext};
use bevy::prelude::*;
use geo::{BooleanOps, Centroid, Coord, CoordsIter, LineString, MultiPolygon, Scale, Translate, TriangulateEarcut, TriangulateSpade, Vector2DOps};
use std::collections::VecDeque;
use std::io;
use geo::triangulate_spade::SpadeTriangulationConfig;
use itertools::Itertools;
use thiserror::Error;

#[derive(Asset, Debug, Reflect, Clone)]
pub struct TileGridAsset {
    pub grid: Grid<Tile>,
    pub collider: Vec<([Vec2; 3], Vec2)>,
}

impl TileGridAsset {
    pub fn new(grid: Grid<Tile>, tile_size: f32) -> Self {
        let polygons = grid_to_polys(&grid, tile_size);

        let tris = 
        polygons.constrained_triangulation(SpadeTriangulationConfig::default()).unwrap()
            .into_iter()
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

fn removed_in_line(line: &geo::LineString<f32>) -> geo::LineString<f32> {
    if line.coords_count() < 3 { return line.clone(); }
    
    let mut points = line.coords().copied();
    let (mut a, mut b) = points.next_tuple().unwrap();
    let mut kept = vec![];
    
    fn along_path(a: Coord<f32>, b: Coord<f32>, c: Coord<f32>) -> bool {
        let dir_1 = (c - b).try_normalize().unwrap_or_default();
        let dir_2 = (b - a).try_normalize().unwrap_or_default();
        dir_1.dot_product(dir_2) >= 0.9995
    }
    // maybe add first one
    {
        let mut coords = line.coords().copied();
        let b = coords.next().unwrap();
        let c = coords.next().unwrap();
        let _ = coords.next_back().unwrap();  // last coord is a duplicate
        let a = coords.next_back().unwrap();
        if !along_path(a, b, c) {
            kept.push(b);
        }
    }
    
    for p in points {
        if !along_path(a, b, p) {
            kept.push(b);
        }
        (a, b) = (b, p);
    }
    LineString::<f32>::new(kept)
}

pub fn grid_to_polys(grid: &Grid<Tile>, tile_size: f32) -> geo::MultiPolygon<f32> {
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

    let polys = divide_reduce(polygons, |a, b| a.union(&b))
        .unwrap_or(MultiPolygon::new(vec![]));
    
    polys.into_iter()
        .map(|p| {
            geo::Polygon::new(
                removed_in_line(p.exterior()),
                p.interiors().into_iter()
                    .map(removed_in_line)
                    .collect()
            )
        })
        .collect()
}
// fn spawn_polygon(polygon: MultiPolygon)

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

pub fn spawn_colliders(
    mut commands: Commands,
    mut tile_grid: EventReader<TileGridLoadEvent>,
) {
    for TileGridLoadEvent(grid, _, parent) in tile_grid.read() {
        commands.entity(*parent).with_children(|parent| {
            // let polys = grid_to_polys(&grid.grid, 16.0);
            // for p in polys {
            //     let vertices = p
            //         .exterior_coords_iter()
            //         .map(|p| Vec2::new(p.x, p.y))
            //         .collect::<Vec<_>>();
            //     parent.spawn((
            //         Name::new("Polygon Collider (Exterior)"),
            //         Collider::polyline(vertices, None),
            //         RigidBody::Static,
            //         SpatialBundle::default(),
            //     ));
            //     for interior in p.interiors() {
            //         let vertices = interior
            //             .exterior_coords_iter()
            //             .map(|p| Vec2::new(p.x, p.y))
            //             .collect::<Vec<_>>();
            //         parent.spawn((
            //             Name::new("Polygon Collider (Interior)"),
            //             Collider::polyline(vertices, None),
            //             RigidBody::Static,
            //             SpatialBundle::default(),
            //         ));
            //     }
            // }

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
