use avian2d::prelude::*;
use bevy::color;
use bevy::prelude::*;
use crate::mouse::MouseCoords;
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
    pub joint: Entity,
}

#[derive(Component, Default, Debug, Reflect, PartialEq, Copy, Clone)]
pub enum WebState {
    #[default]
    Idle,
    Firing(Dir2),
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

                    *web_state = WebState::Firing(Dir2::new_unchecked(dir));
                }
            }
            WebState::Firing(_) => {
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
    mut q_joint: Query<&mut DistanceJoint>,
    spatial_query: SpatialQuery,
    time: Res<Time>,
) {
    for (source, stats, mut state, mut transform) in query.iter_mut() {
        let WebState::Firing(dir) = *state else {
            continue;
        };

        if let Some(hit) = spatial_query.cast_shape(
            &Collider::circle(stats.radius),
            transform.translation.truncate(),
            0.0,
            dir,
            time.delta_seconds() * stats.travel_speed,
            false,
            SpatialQueryFilter::default().with_excluded_entities([source.player]),
        ) {
            let mut joint = q_joint.get_mut(source.joint).unwrap();
            let cur = q_position.get(hit.entity).unwrap().translation();
            joint.rest_length = Vec3::distance(cur, transform.translation);
            let offset = hit.point1 - cur.truncate();

            *state = WebState::Attached {
                swing: false,
                pull: false,
                offset,
                target: hit.entity,
            };
        } else {
            transform.translation += dir.xy().extend(0.0) * (time.delta_seconds() * stats.travel_speed);
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
            WebState::Firing(_) => {}
            WebState::Attached { target, offset, .. } => {
                let target = position.get(*target).unwrap().translation();
                transform.translation = target + offset.extend(0.0);
            }
        }
    }
}

fn gizmos_web(
    mut gizmos: Gizmos,
    query: Query<(&GlobalTransform, &WebStats, &WebSource, &WebState)>,
    pos: Query<&GlobalTransform>,
) {
    for (transform, stats, source, state) in query.iter() {
        let source_pos = pos.get(source.player).unwrap();

        gizmos.line_2d(
            transform.translation().truncate(),
            source_pos.translation().truncate(),
            Color::linear_rgb(0., 1.0, 0.0));

        gizmos.circle_2d(
            transform.translation().truncate(),
            stats.radius,
            match state {
                WebState::Idle => Color::WHITE,
                WebState::Firing(_) => Color::linear_rgb(0.0, 0.0, 0.0),
                WebState::Attached { .. } => Color::linear_rgb(1.0, 0.0, 0.0),
            },
        );

        match state {
            WebState::Idle => {}
            WebState::Firing(dir) => {
                gizmos.ray_2d(
                    transform.translation().truncate(),
                    dir.xy() * stats.travel_speed,
                    Color::linear_rgb(0.0, 1.0, 1.0),
                );
            }
            WebState::Attached { .. } => {}
        }
    }
}

fn handle_joint(
    mut player: Query<&mut DistanceJoint>,
    web: Query<(&WebState, &WebSource), Changed<WebState>>,
    positions: Query<&GlobalTransform>,
) {
    for (state, source) in web.iter() {
        let mut joint = player.get_mut(source.joint).unwrap();
        match *state {
            WebState::Idle | WebState::Firing(_) => {
                joint.entity1 = joint.entity2;
            }
            WebState::Attached { target, pull, swing, offset } => {
                joint.entity1 = target;
                let player_pos = positions.get(source.player).unwrap().translation();
                let target_pos = positions.get(target).unwrap().translation();
                joint.local_anchor1 = offset;
                joint.rest_length = Vec3::distance(player_pos, target_pos);
                joint.length_limits = if swing {
                    Some(DistanceLimit::new(0.0, joint.rest_length))
                } else {
                    Some(DistanceLimit::new(0.0, f32::INFINITY))
                };

                // joint.rest_length = 0.0;

                info!("{:?} {:?}", joint.length_limits, joint.rest_length);
            }
        }
    }
}

fn animate_web() {}
