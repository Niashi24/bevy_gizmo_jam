use bevy::app::App;
use bevy::prelude::*;

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_computed_state::<InGame>()
            .add_computed_state::<Paused>()
            .add_sub_state::<GamePhase>()
            .enable_state_scoped_entities::<AppState>()
            .enable_state_scoped_entities::<InGame>()
            .enable_state_scoped_entities::<Paused>();
    }
}

#[derive(States, Default, Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum AppState {
    #[default]
    Loading,
    Menu,
    Game(GameInfo),
}

#[derive(Default, Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub struct GameInfo {
    pub level: i32,
    pub paused: bool,
}

impl GameInfo {
    pub fn toggle_paused(&mut self) {
        self.paused = !self.paused;
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct InGame;

impl ComputedStates for InGame {
    type SourceStates = AppState;

    fn compute(sources: Self::SourceStates) -> Option<Self> {
        match sources {
            AppState::Game { .. } => Some(InGame),
            _ => None,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Paused(pub bool);

impl ComputedStates for Paused {
    type SourceStates = AppState;

    fn compute(sources: Self::SourceStates) -> Option<Self> {
        match sources {
            AppState::Game(GameInfo { paused, .. }) => Some(Self(paused)),
            _ => None,
        }
    }
}

#[derive(SubStates, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[source(InGame = InGame)]
pub enum GamePhase {
    #[default]
    Setup,
    InGame,
    Completed
}