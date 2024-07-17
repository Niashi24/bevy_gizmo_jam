use bevy::prelude::*;
use std::ops::{Add, Mul, Sub};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CameraRegion2d>()
            .register_type::<CameraTarget>()
            .add_systems(Update, follow_target);
    }
}

#[derive(Component, Reflect)]
pub struct CameraRegion2d(pub Rect);

#[derive(Component, Reflect)]
pub struct CameraTarget(pub Option<Entity>);

pub fn follow_target(
    mut query: Query<(&mut Transform, &CameraTarget, Option<&CameraRegion2d>)>,
    position: Query<&GlobalTransform>,
    time: Res<Time>,
) {
    for (mut transform, target, region) in query.iter_mut() {
        let &CameraTarget(Some(entity)) = target else {
            continue;
        };

        let Ok(target) = position.get(entity) else {
            continue;
        };

        let mut target_pos = nudge(
            transform.translation.xy(),
            target.translation().xy(),
            8.0,
            time.delta_seconds(),
        );

        if let Some(&CameraRegion2d(region)) = region {
            target_pos.x = target_pos.x.clamp(region.min.x, region.max.x);
            target_pos.y = target_pos.y.clamp(region.min.y, region.max.y);
        }

        transform.translation = target_pos.extend(transform.translation.z);
    }
}

pub fn nudge<N, F>(a: N, b: N, decay: F, delta: F) -> N
where
    N: Mul<F, Output = N> + Add<N, Output = N> + Sub<N, Output = N> + Copy,
    F: num_traits::real::Real,
{
    b + (a - b) * F::exp(-decay * delta)
}
