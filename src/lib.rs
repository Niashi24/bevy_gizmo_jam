#![allow(clippy::type_complexity)]

mod audio;
mod loading;
mod menu;
mod player;
mod state;
mod pause;
mod tileset;
mod camera;

// use crate::audio::InternalAudioPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::player::PlayerPlugin;

use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_kira_audio::AudioPlugin;
use crate::pause::PausePlugin;
use crate::state::{AppState, InGame, StatesPlugin};
use crate::tileset::TilePlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                AudioPlugin,
                StatesPlugin,
                LoadingPlugin,
                MenuPlugin,
                PlayerPlugin,
                PausePlugin,
                TilePlugin,
            ))
            .add_systems(Update, log_app_state);

        #[cfg(debug_assertions)]
        {
            app.add_plugins((
                // FrameTimeDiagnosticsPlugin,
                LogDiagnosticsPlugin::default(),
                WorldInspectorPlugin::default(),
            ));
        }
    }
}

fn log_app_state(state: Res<State<AppState>>) {
    if state.is_changed() {
        // dbg!(state.get());
    }
}
