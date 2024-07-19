#![allow(clippy::type_complexity)]

use avian2d::prelude::*;
use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_kira_audio::AudioPlugin;
use bevy_tnua::prelude::TnuaControllerPlugin;
use bevy_tnua_avian2d::TnuaAvian2dPlugin;
use web::WebPlugin;

use crate::camera::CameraPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::mouse::MousePlugin;
use crate::pause::PausePlugin;
use crate::player::PlayerPlugin;
use crate::state::{AppState, StatesPlugin};
use crate::tileset::TilePlugin;

mod audio;
mod camera;
mod loading;
mod menu;
mod pause;
mod player;
mod state;
mod tileset;
mod web;
mod mouse;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PhysicsPlugins::default(),
            TnuaAvian2dPlugin::default(),
            TnuaControllerPlugin::default(),
            AudioPlugin,
            StatesPlugin,
            LoadingPlugin,
            MenuPlugin,
            PlayerPlugin,
            PausePlugin,
            TilePlugin,
            CameraPlugin,
            WebPlugin,
            MousePlugin,
        ))
        .add_systems(Update, log_app_state);

        app.insert_resource::<Gravity>(Gravity(Vec2::Y * -160.0));

        #[cfg(debug_assertions)]
        {
            app.add_plugins((
                // FrameTimeDiagnosticsPlugin,
                LogDiagnosticsPlugin::default(),
                WorldInspectorPlugin::default(),
                PhysicsDebugPlugin::default(),
            ));
        }
    }
}

fn log_app_state(state: Res<State<AppState>>) {
    if state.is_changed() {
        // dbg!(state.get());
    }
}
