use std::ops::{Add, Mul, Sub};
use bevy::prelude::*;
use crate::player::Player;
use crate::tileset::load::TileGridAsset;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        
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
    for (mut transform, target, region) in query.iter_mut()  {
        let &CameraTarget(Some(entity)) = target else {
            continue;
        };
        
        let Ok(target) = position.get(entity) else {
            continue;
        };
        
        transform.translation = nudge(
            transform.translation,
            target.translation(),
            8.0,
            time.delta_seconds(),
        );
    }
}

// pub fn spawn_camera(
//     mut commands: Commands,
//     player: Query<Entity, With<Player>>,
//     grid: Query<&Handle<TileGridAsset>>,
//     grid_assets: Res<Assets<TileGridAsset>>,
// ) {
//     
// }
pub fn nudge<N, F>(a: N, b: N, decay: F, delta: F) -> N
where
    N: Mul<F, Output=N> + Add<N, Output=N> + Sub<N, Output=N> + Copy,
    F: num_traits::real::Real,
{
    b + (a - b) * F::exp(-decay * delta)
}