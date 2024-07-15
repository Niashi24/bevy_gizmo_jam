use crate::loading::TextureAssets;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use crate::state::{AppState, InGame, Paused};

pub struct PlayerPlugin;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(InGame), spawn_player);
        //     .add_systems(Update, move_player.run_if(in_state(Paused(false))));
    }
}

fn spawn_player(
    mut commands: Commands,
    assets: Res<TextureAssets>,
) {
    commands.spawn((
        Name::new("Camera"),
        StateScoped(InGame),
        Camera2dBundle {
            projection: OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical(240.0),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 10.0),
            ..default()
        },
    ));
    
    commands.spawn((
        Name::new("Player"),
        StateScoped(InGame),
        SpriteBundle {
            texture: assets.player.clone(),
            ..default()
        }
    ));
}