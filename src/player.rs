use crate::camera::{CameraRegion2d, CameraTarget};
use crate::loading::{Levels, TextureAssets};
use crate::state::{AppState, InGame, Paused};
use crate::tileset;
use crate::tileset::load::{TileGridBundle, TileGridLoadEvent, TileGridSettings};
use crate::tileset::tile::Tile;
use crate::web::{WebBundle, WebSource, WebState, WebStats};
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
        )
            .register_type::<PlayerWeb>();
        //     .add_systems(Update, move_player.run_if(in_state(Paused(false))));
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component, Clone)]
pub struct PlayerStats {
    pub speed: f32,
    pub air_multiplier: f32,
    pub walk: TnuaBuiltinWalk,
}

#[derive(Component, Copy, Clone, Debug, Reflect)]
pub struct PlayerWeb(pub Entity);

fn spawn_level(mut commands: Commands, assets: Res<TextureAssets>, levels: Res<Levels>) {
    
    // dbg!(&levels.level_map);
    commands.spawn((
        Name::new("Tilemap"),
        StateScoped(InGame),
        TileGridBundle {
            settings: TileGridSettings {
                solid_texture: assets.block.clone(),
                ramp_texture: assets.ramp.clone(),
                tile_size: 16.0,
            },
            tile_grid: levels.level_map.get("levels/level-fall.png").unwrap().clone(),
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
        let scale = 0.9;
        let sensor = Collider::circle(radius * scale);
        info!("{:?}", &collider);
        info!("{:?}", &sensor);

        let player = commands
            .spawn((
                Name::new("Player"),
                StateScoped(InGame),
                Player,
                PlayerStats {
                    speed: 128.0,
                    air_multiplier: 0.5,
                    walk: TnuaBuiltinWalk {
                        float_height: 6.0,
                        acceleration: 256.0,
                        air_acceleration: 0.0,
                        cling_distance: 0.5,
                        ..default()
                    },
                },
                SpriteBundle {
                    transform: Transform::from_translation(pos),
                    texture: texture_assets.player.clone(),
                    ..default()
                },
                MassPropertiesBundle::new_computed(&collider, 1.0),
                collider,
                TnuaAvian2dSensorShape(sensor),
                TnuaControllerBundle::default(),
                RigidBody::Dynamic,
                LockedAxes::ROTATION_LOCKED,
                Restitution::new(0.0).with_combine_rule(CoefficientCombine::Min),
                // ExternalForce::new(Vec2::ZERO).with_persistence(false),
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
        
        let mut rect = Rect::from_corners(grid_anchor.xy(), bottom_right);
        rect.min += Vec2::new(-1.0, 1.0) * settings.tile_size / 2.0;
        rect.max += Vec2::new(-1.0, 1.0) * settings.tile_size / 2.0;

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
            CameraRegion2d(rect),
            CameraTarget(Some(player)),
        ));

        // let joint = commands.spawn((
        //     Name::new("Joint"),
        //     StateScoped(InGame),
        //     DistanceJoint::new(player, player)
        //         .with_rest_length(100.0)
        //         .with_linear_velocity_damping(0.1)
        //         .with_angular_velocity_damping(1.0)
        //         .with_compliance(0.00000001),
        // )).id();

        let web = commands.spawn((
            Name::new("Web"),
            StateScoped(InGame),
            WebBundle {
                web_source: WebSource { player, joint: None},
                web_state: WebState::default(),
                web_stats: WebStats {
                    pull_force: 96000.0,
                    travel_speed: 640.0,
                    radius: 2.0,
                },
            },
            SpatialBundle::default(),
        )).id();
        
        commands.entity(player)
            .insert(PlayerWeb(web));
    }
}

pub fn move_player(
    mut player: Query<(&mut TnuaController, &mut LinearVelocity, &PlayerStats, &PlayerWeb, &GlobalTransform)>,
    web: Query<&WebState>,
    input: Res<ButtonInput<KeyCode>>,
    q_pos: Query<&GlobalTransform>,
    time: Res<Time>,
) {
    for (mut controller, mut vel, stats, p_web, transform) in player.iter_mut() {
        let x =
            input.pressed(KeyCode::KeyD) as i32 as f32 - input.pressed(KeyCode::KeyA) as i32 as f32;

        controller.basis(TnuaBuiltinWalk {
            desired_forward: Vec3::X * x,
            desired_velocity: Vec3::X * (x * stats.speed),
            ..stats.walk.clone()
        });
        
        if !controller.is_airborne().is_ok_and(|x| !x ) {
            let web_state = web.get(p_web.0).unwrap();
            match web_state {
                WebState::Attached { target, offset, .. } => {
                    let p_1 = transform.translation().truncate();
                    let p_2 = q_pos.get(*target).unwrap().translation().truncate();
                    
                    let mut dir = (p_2 - p_1).normalize_or_zero();
                    // rotate 90deg clockwise to get tangent
                    dir = (Rot2::FRAC_PI_2.inverse()) * dir;
                    vel.0 += dir * (time.delta_seconds() * x * stats.speed * stats.air_multiplier);
                }
                _ => {}
            }
        }
    }
}
