use avian2d::prelude::*;
use bevy::color;
use bevy::color::palettes::css::RED;
use bevy::prelude::*;
use crate::mouse::MouseCoords;
use crate::state::GamePhase::InGame;
use crate::state::Paused;

pub struct WebPlugin;

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct WebSystemSet;

impl Plugin for WebPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(OnEnter(AppState::Web), web_setup);
        app
            .register_type::<WebState>()
            .register_type::<WebStats>()
            .register_type::<WebSource>()
            .add_systems(
                Update,
                ((
                     handle_input,
                     (
                         move_and_attach_web,  // Moving
                         keep_web_attached,  // Attached or Idle
                     ),
                     handle_joint,
                 ).chain().in_set(WebSystemSet),
                 gizmos_web,
                ))
            .configure_sets(Update, WebSystemSet.run_if(in_state(Paused(false))));
    }
}

#[derive(Bundle, Debug)]
pub struct WebBundle {
    pub web_source: WebSource,
    pub web_state: WebState,
    pub web_stats: WebStats,
}

#[derive(Component, Debug, Reflect)]
pub struct WebSource {
    pub player: Entity,
    pub joint: Option<Entity>,
}

#[derive(Component, Default, Debug, Reflect, PartialEq, Copy, Clone)]
pub enum WebState {
    #[default]
    Idle,
    Firing {
        pos: Vec2,
        dir: Dir2,
    },
    Attached {
        swing: bool,
        pull: bool,
        offset: Vec2,
        // state: WebAttachState,
        target: Entity,
    },
}

#[derive(Default, Debug, Reflect, PartialEq, Eq)]
pub enum WebAttachState {
    #[default]
    Fall,
    Swing,
    Pull,
    Charge,
}

#[derive(Component, Debug, Reflect, PartialEq, Copy, Clone)]
pub struct WebStats {
    pub pull_force: f32,
    pub travel_speed: f32,
    pub radius: f32,
}

pub fn handle_input(
    mut query: Query<(&mut WebState, &mut Transform, &WebSource)>,
    q_position: Query<&GlobalTransform>,
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mouse_pos: Res<MouseCoords>,
) {
    let pull = key_input.pressed(KeyCode::Space);
    let swing = key_input.pressed(KeyCode::ShiftLeft);
    let throw = mouse_input.any_pressed([MouseButton::Left, MouseButton::Right]);

    for (mut web_state, mut transform, source) in query.iter_mut() {
        match web_state.as_mut() {
            WebState::Idle => {
                if throw {
                    let cur = q_position.get(source.player).unwrap().translation();
                    transform.translation = cur;
                    let cur = cur.truncate();
                    let dir = (mouse_pos.0 - cur).try_normalize().unwrap_or(Vec2::X);

                    *web_state = WebState::Firing {
                        pos: cur,
                        dir: Dir2::new_unchecked(dir),
                    };
                }
            }
            WebState::Firing { .. } => {
                if !throw {
                    *web_state = WebState::Idle;
                }
            }
            WebState::Attached { pull: p_pull, swing: p_swing, target: _target, offset } => {
                if !throw {
                    *web_state = WebState::Idle;
                    continue;
                }

                *p_pull = pull;
                *p_swing = swing;

                // if new_state != *state {
                //     *state = new_state;
                // }
            }
        }
    }
}

fn move_and_attach_web(
    mut query: Query<(&WebSource, &WebStats, &mut WebState, &mut Transform)>,
    q_position: Query<&GlobalTransform>,
    spatial_query: SpatialQuery,
    time: Res<Time>,
) {
    for (source, stats, mut state, mut transform) in query.iter_mut() {
        let WebState::Firing { pos, dir } = state.as_mut() else {
            continue;
        };

        if let Some(hit) = spatial_query.cast_shape(
            &Collider::circle(stats.radius),
            *pos,
            0.0,
            *dir,
            time.delta_seconds() * stats.travel_speed,
            false,
            SpatialQueryFilter::default().with_excluded_entities([source.player]),
        ) {
            let cur = q_position.get(hit.entity).unwrap().translation();
            let offset = hit.point1 - cur.truncate();

            *state = WebState::Attached {
                swing: false,
                pull: false,
                offset,
                target: hit.entity,
            };
        } else {
            *pos += dir.xy() * (time.delta_seconds() * stats.travel_speed);
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
        }
    }
}

fn keep_web_attached(
    mut webs: Query<(&mut Transform, &WebSource, &WebState)>,
    position: Query<&GlobalTransform>,
) {
    for (mut transform, source, state) in webs.iter_mut() {
        match state {
            WebState::Idle => {}
            WebState::Firing { .. } => {}
            WebState::Attached { target, offset, .. } => {
                let target = position.get(*target).unwrap().translation();
                transform.translation = target + offset.extend(0.0);
            }
        }
    }
}

fn gizmos_web(
    mut gizmos: Gizmos,
    query: Query<(&WebStats, &WebSource, &WebState)>,
    pos: Query<&GlobalTransform>,
) {
    for (stats, source, state) in query.iter() {
        let source_pos = pos.get(source.player).unwrap();

        match *state {
            WebState::Idle => {}
            WebState::Firing { pos, dir } => {
                gizmos.ray_2d(
                    pos,
                    dir.xy() * stats.travel_speed,
                    Color::linear_rgb(0.0, 1.0, 1.0),
                );
                
                gizmos.circle_2d(
                    pos,
                    stats.radius,
                    Color::BLACK,
                );
            }
            WebState::Attached { swing, pull, offset, target } => {
                if swing {
                    continue;
                }
                
                gizmos.line_2d(
                    source_pos.translation().truncate(),
                    pos.get(target).unwrap().translation().truncate() + offset,
                    match pull {
                        true => color::palettes::basic::GREEN,
                        false => color::palettes::basic::BLUE,
                    }
                )
            }
        }
    }
}

fn spawn_joint(
    commands: &mut Commands,
    offset: Vec2,
    player: Entity,
    anchor: Entity,
    distance: f32,
) -> Entity {
    commands.spawn((
        Name::new("Joint"),
        StateScoped(InGame),
        DistanceJoint::new(player, anchor)
            .with_local_anchor_2(offset)
            .with_rest_length(distance)
            .with_limits(0.0, distance)
            .with_linear_velocity_damping(0.0)
            .with_angular_velocity_damping(0.0)
            .with_compliance(0.00000001),
    )).id()
}

fn handle_joint(
    mut q_joint: Query<&mut DistanceJoint>,
    mut web: Query<(&WebState, &mut WebSource, &WebStats)>,
    positions: Query<&GlobalTransform>,
    mut forces: Query<&mut ExternalImpulse>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (state, mut source, stats) in web.iter_mut() {
        // let mut joint = player.get_mut(source.joint).unwrap();
        match *state {
            WebState::Idle | WebState::Firing { .. } => {
                if let Some(joint) = source.joint.and_then(|e| commands.get_entity(e)) {
                    joint.despawn_recursive();
                }
            }
            WebState::Attached { target, pull, swing, offset } => {
                let p_1 = positions.get(source.player).unwrap().translation().truncate();
                let p_2 = positions.get(target).unwrap().translation().truncate() + offset;
                
                match (swing, source.joint) {
                    (true, None) => {                        
                        let joint = spawn_joint(
                            &mut commands,
                            offset,
                            source.player,
                            target,
                            Vec2::distance(p_1, p_2),
                        );
                        
                        source.joint = Some(joint);
                    }
                    (true, Some(joint)) => {
                        let mut joint = q_joint.get_mut(joint).unwrap();
                        // handle distance
                        joint.rest_length = Vec2::distance(p_1, p_2);
                        joint.length_limits = Some(DistanceLimit::new(0.0, joint.rest_length));
                        
                    }
                    (false, Some(joint)) => {
                        if let Some(joint) = commands.get_entity(joint) {
                            joint.despawn_recursive();
                        }
                        
                        source.joint = None;
                    }
                    (false, None) => {}
                };
                
                // handle pull
                if pull {
                    let mut player = forces.get_mut(source.player).unwrap();
                    let a_to_b = (p_2 - p_1).normalize_or_zero();
                    let applied = a_to_b * stats.pull_force * time.delta_seconds();

                    player.apply_impulse(applied);

                    if let Ok(mut anchor) = forces.get_mut(target) {
                        anchor.apply_impulse_at_point(
                            -applied,
                            offset,
                            Vec2::ZERO
                        );
                    }
                }
                // joint.entity1 = target;
                // let player_pos = positions.get(source.player).unwrap().translation();
                // let target_pos = positions.get(target).unwrap().translation();
                // joint.local_anchor1 = offset;
                // joint.rest_length = Vec3::distance(player_pos, target_pos);
                // joint.length_limits = if swing {
                //     Some(DistanceLimit::new(0.0, joint.rest_length))
                // } else {
                //     Some(DistanceLimit::new(0.0, f32::INFINITY))
                // };
                // 
                // // joint.rest_length = 0.0;
                // 
                // info!("{:?} {:?}", joint.length_limits, joint.rest_length);
            }
        }
    }
}

fn animate_web() {}
