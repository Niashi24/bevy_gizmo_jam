use crate::camera::{CameraRegion2d, CameraTarget};
use crate::loading::{Levels, TextureAssets};
use crate::state::{AppState, InGame, Paused};
use crate::tileset;
use crate::tileset::load::{TileGridBundle, TileGridLoadEvent, TileGridSettings};
use crate::tileset::tile::Tile;
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy_tnua::prelude::{TnuaBuiltinWalk, TnuaController, TnuaControllerBundle};
use bevy_tnua_avian2d::TnuaAvian2dSensorShape;

pub struct PlayerPlugin;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(InGame), spawn_level).add_systems(
            Update,
            (
                spawn_player_and_camera
                    .after(tileset::load::spawn_grid)
                    .run_if(not(in_state(AppState::Loading))),
                move_player.run_if(in_state(Paused(false))),
            ),
        );
        //     .add_systems(Update, move_player.run_if(in_state(Paused(false))));
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component, Clone)]
pub struct PlayerStats {
    pub speed: f32,
    pub walk: TnuaBuiltinWalk,
}

fn spawn_level(mut commands: Commands, assets: Res<TextureAssets>, levels: Res<Levels>) {
    commands.spawn((
        Name::new("Tilemap"),
        StateScoped(InGame),
        TileGridBundle {
            settings: TileGridSettings {
                solid_texture: assets.block.clone(),
                ramp_texture: assets.ramp.clone(),
                tile_size: 16.0,
            },
            tile_grid: levels.test_level.clone(),
            ..default()
        },
    ));
}

fn spawn_player_and_camera(
    mut commands: Commands,
    mut tile_grid: EventReader<TileGridLoadEvent>,
    texture_assets: Res<TextureAssets>,
    global_pos: Query<&GlobalTransform>,
) {
    for TileGridLoadEvent(grid, settings, parent) in tile_grid.read() {
        let grid_anchor = global_pos.get(*parent).unwrap().translation();

        let Some((p_x, p_y)) = grid
            .0
            .iter()
            .filter_map(|((x, y), item)| match item {
                Tile::Player => Some((x, y)),
                _ => None,
            })
            .next()
        else {
            continue;
        };

        let mut pos = grid_anchor;
        pos.x += p_x as f32 * settings.tile_size;
        pos.y -= p_y as f32 * settings.tile_size;

        let radius = 6.0;
        let collider = Collider::circle(radius);
        let scale = 0.5;
        let sensor = Collider::circle(radius * scale);
        info!("{:?}", &collider);
        info!("{:?}", &sensor);

        // commands.spawn((
        //     Name::new("Player"),
        //     StateScoped(InGame),
        //     Player,
        //     PlayerStats {
        //         speed: 128.0,
        //         walk: TnuaBuiltinWalk {
        //             float_height: 3.0,
        //             acceleration: 256.0,
        //             air_acceleration: 16.0,
        //             ..default()
        //         },
        //     },
        //     SpriteBundle {
        //         transform: Transform::from_translation(pos),
        //         texture: texture_assets.player.clone(),
        //         ..default()
        //     },
        //     sensor.clone(),
        //     TnuaAvian2dSensorShape(sensor.clone()),
        //     TnuaControllerBundle::default(),
        //     RigidBody::Dynamic,
        //     LockedAxes::ROTATION_LOCKED,
        //     Restitution::new(0.0).with_combine_rule(CoefficientCombine::Min),
        // ));

        let player = commands
            .spawn((
                Name::new("Player"),
                StateScoped(InGame),
                Player,
                PlayerStats {
                    speed: 128.0,
                    walk: TnuaBuiltinWalk {
                        float_height: 6.0,
                        acceleration: 256.0,
                        air_acceleration: 16.0,
                        ..default()
                    },
                },
                SpriteBundle {
                    transform: Transform::from_translation(pos),
                    texture: texture_assets.player.clone(),
                    ..default()
                },
                collider,
                TnuaAvian2dSensorShape(sensor),
                TnuaControllerBundle::default(),
                RigidBody::Dynamic,
                LockedAxes::ROTATION_LOCKED,
                Restitution::new(0.0).with_combine_rule(CoefficientCombine::Min),
            ))
            .id();

        // let bounds = CameraRegion2d(Rec)
        let mut bottom_right = grid_anchor.xy();
        bottom_right.x += grid.0.w as f32 * settings.tile_size;
        bottom_right.y -= grid.0.h as f32 * settings.tile_size;

        let mut center = grid_anchor;

        center.x += grid.0.w as f32 * settings.tile_size / 2.0;
        center.y -= grid.0.h as f32 * settings.tile_size / 2.0;
        center.z += 10.0;

        commands.spawn((
            Name::new("Camera"),
            StateScoped(InGame),
            Camera2dBundle {
                transform: Transform::from_translation(center),
                projection: OrthographicProjection {
                    scaling_mode: ScalingMode::FixedVertical(240.0),
                    ..default()
                },
                ..default()
            },
            CameraRegion2d(Rect::from_corners(grid_anchor.xy(), bottom_right)),
            CameraTarget(Some(player)),
        ));
    }
}

fn move_player(
    mut player: Query<(&mut TnuaController, &PlayerStats)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (mut controller, stats) in player.iter_mut() {
        let x = input.pressed(KeyCode::KeyD) as i32 as f32
            - input.pressed(KeyCode::KeyA) as i32 as f32;

        controller.basis(TnuaBuiltinWalk {
            desired_forward: Vec3::X * x,
            desired_velocity: Vec3::X * (x * stats.speed),
            ..stats.walk.clone()
        });
    }
}
