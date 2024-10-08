use bevy::prelude::*;
use std::ops::{Add, Mul, Sub};
use avian2d::prelude::{Collider, PhysicsSet};
use bevy::color::palettes::basic::RED;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CameraRegion2d>()
            .register_type::<CameraTarget>()
            .add_systems(PostUpdate, follow_target
                .after(PhysicsSet::Sync)
                .before(TransformSystem::TransformPropagate))
            .add_systems(Update, polyline_gizmos);
    }
}

fn polyline_gizmos(mut gizmos: Gizmos, q_poly: Query<&Collider>) {
    for poly in q_poly.iter() {
        if let Some(line) = poly.shape().as_polyline() {
            for p in line.vertices() {
                gizmos.circle_2d((*p).into(), 1.0, RED);
            }
        }
    }
}

#[derive(Component, Reflect)]
pub struct CameraRegion2d(pub Rect);

#[derive(Component, Reflect)]
pub struct CameraTarget(pub Option<Entity>);

pub fn follow_target(
    mut query: Query<(&mut Transform, &CameraTarget, Option<&CameraRegion2d>, &OrthographicProjection)>,
    position: Query<&GlobalTransform>,
    time: Res<Time>,
) {
    for (mut transform, target, region, ortho) in query.iter_mut() {
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

        if let Some(&CameraRegion2d(mut region)) = region {
            region.min -= ortho.area.min;
            region.max -= ortho.area.max;
            
            // target_pos.x = target_pos.x.max(region.min.x).min(region.max.x);
            // target_pos.y = target_pos.y.max(region.min.y).min(region.max.y);
            
            target_pos.x = if region.min.x < region.max.x {
                target_pos.x.clamp(region.min.x, region.max.x)
            } else {
                (region.min.x + region.max.x) / 2.0
            };
            target_pos.y = if region.min.y < region.max.y {
                target_pos.y.clamp(region.min.y, region.max.y)
            } else {
                (region.min.y + region.max.y) / 2.0
            };
            // target_pos.y = target_pos.y.clamp(region.min.y, region.max.y);
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
