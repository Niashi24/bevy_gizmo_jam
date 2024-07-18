use avian2d::prelude::*;
use bevy::prelude::*;

pub struct WebPlugin;

impl Plugin for WebPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(OnEnter(AppState::Web), web_setup);
        app
            .register_type::<WebState>()
            .register_type::<WebStats>()
            .register_type::<WebSource>()
            .add_systems(Update, handle_input);
    }
}

#[derive(Bundle, Debug)]
pub struct WebBundle {
    pub web_source: WebSource,
    pub web_state: WebState,
    pub web_stats: WebStats,
}

#[derive(Component, Debug, Reflect)]
pub struct WebSource(pub Entity);

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
    mut Query: Query<&mut WebState>,
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    let pull = key_input.pressed(KeyCode::Space);
    let swing = key_input.pressed(KeyCode::ShiftLeft);
    let throw = mouse_input.any_pressed([MouseButton::Left, MouseButton::Right]);

    for mut web_state in Query.iter_mut() {
        match *web_state {
            WebState::Idle => {
                if throw {
                    *web_state = WebState::Firing(Dir2::X);
                }
            },
            WebState::Firing(_) => {
                if !throw {
                    *web_state = WebState::Idle;
                }
            },
            WebState::Attached { pull: p_pull, swing: p_swing, target: _target, offset } => {
                if !throw {
                    *web_state = WebState::Idle;
                    continue;
                }

                let new_state = match (pull, swing) {
                    (true, true) => WebAttachState::Charge,
                    (true, false) => WebAttachState::Pull,
                    (false, true) => WebAttachState::Swing,
                    (false, false) => WebAttachState::Fall,
                };

                // if new_state != *state {
                //     *state = new_state;
                // }
            },
        }
    }
}

fn set_initial_position(
    mut query: Query<(&WebState, &WebSource, &mut Transform), Changed<WebState>>,
    pos: Query<&GlobalTransform>,
) {
    for (state, source, mut transform) in query.iter_mut() {
        if matches!(*state, WebState::Firing(_)) {
            let pos = pos.get(source.0).unwrap();
            transform.translation = pos.translation();
        }
    }
}

fn move_and_attach_web(
    mut query: Query<(&WebSource, &WebStats, &mut WebState, &mut Transform)>,
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
            SpatialQueryFilter::default().with_excluded_entities([source.0]),
        ) {
            *state = WebState::Attached {
                swing: false,
                pull: false,
                offset: hit.point1,
                target: hit.entity,
            };
        }
    }
}

fn gizmos_web(
    mut gizmos: Gizmos,
) {
    
}

fn handle_joint(
    mut player: Query<&mut DistanceJoint>,
    web: Query<(&WebState, &WebSource), Changed<WebState>>,
    positions: Query<&GlobalTransform>,
) {
    for (state, source) in web.iter() {
        let mut joint = player.get_mut(source.0).unwrap();
        match *state {
            WebState::Idle | WebState::Firing(_) => {
                joint.entity2 = joint.entity1;
            },
            WebState::Attached { target, .. } => {
                joint.entity2 = target;
                let player_pos = positions.get(source.0).unwrap().translation();
                let target_pos = positions.get(target).unwrap().translation();
                joint.rest_length = Vec3::distance(player_pos, target_pos);
            },
        }
    }
}

fn animate_web() {}
