use bevy::input::common_conditions::{input_just_pressed, input_pressed};
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use crate::state::{AppState, GameInfo, InGame, Paused};

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(InGame), spawn_pause_menu)
            .add_systems(OnEnter(Paused(true)), set_pause_enabled::<true>)
            .add_systems(OnEnter(Paused(false)), set_pause_enabled::<false>)
            .add_systems(Update, (
                toggle_pause.run_if(input_just_pressed(KeyCode::Escape)),
                quit_button.run_if(in_state(Paused(true))),
            ));
    }
}

#[derive(Component)]
pub struct PauseMenu;

#[derive(Component)]
pub struct QuitButton;

fn spawn_pause_menu(
    mut commands: Commands,
) {
    commands.spawn((
        Name::new("Pause Menu"),
        StateScoped(InGame),
        PauseMenu,
        NodeBundle {
            background_color: BackgroundColor(Color::linear_rgba(0.1, 0.1, 0.1, 0.5)),
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                // hide by default
                display: Display::None,
                ..default()
            },
            ..default()
        }))
        .with_children(|parent| {
            parent.spawn((
                NodeBundle {
                    // border_radius: BorderRadius::
                    background_color: BackgroundColor(Color::linear_rgba(0.05, 0.05, 0.05, 0.8)),
                    border_radius: BorderRadius::all(Val::Px(20.0)),
                    style: Style {
                        width: Val::Percent(50.0),
                        height: Val::Auto,
                        // justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                }
            ))
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle::from_section(
                            "Paused",
                            TextStyle {
                                font_size: 30.0,
                                ..default()
                            },
                        ).with_style(Style {
                            margin: UiRect::all(Val::Px(20.0)),
                            ..default()
                        }),
                    ));

                    parent.spawn((
                        QuitButton,
                        ButtonBundle {
                            background_color: BackgroundColor(BUTTON_NORMAL),
                            border_radius: BorderRadius::all(Val::Px(10.0)),
                            style: Style {
                                margin: UiRect::all(Val::Px(10.0)),
                                ..default()
                            },
                            ..default()
                        }
                    )).with_children(|parent| {
                        parent.spawn((
                            TextBundle::from_section(
                                "Quit",
                                TextStyle {
                                    font_size: 30.0,
                                    ..default()
                                },
                            ).with_style(Style {
                                margin: UiRect::all(Val::Px(10.0)),
                                ..default()
                            })
                        ));
                    });
                });
        });
}

fn set_pause_enabled<const ENABLED: bool>(
    mut query: Query<&mut Style, With<PauseMenu>>
) {
    for mut style in query.iter_mut() {
        style.display = const {
            if ENABLED {
                Display::Flex
            } else {
                Display::None
            }
        };
    }
}

fn toggle_pause(
    mut next_state: ResMut<NextState<AppState>>,
    current_state: Res<State<AppState>>,
) {
    let &AppState::Game(mut game) = current_state.get() else {
        return;
    };
    game.toggle_paused();

    next_state.set(AppState::Game(game));
}

const BUTTON_NORMAL: Color = Color::linear_rgba(0.025, 0.025, 0.025, 1.);
const BUTTON_HOVER: Color = Color::linear_rgba(0.05, 0.05, 0.05, 1.);

fn quit_button(
    mut next_state: ResMut<NextState<AppState>>,
    mut button: Query<
        (&mut BackgroundColor, &Interaction),
        (Changed<Interaction>, With<QuitButton>)
    >,
) {
    let Ok((mut color, itn)) = button.get_single_mut() else { return; };
    match *itn {
        Interaction::Pressed => {
            next_state.set(AppState::Menu);
        }
        Interaction::Hovered => {
            color.0 = BUTTON_HOVER;
        }
        Interaction::None => {
            color.0 = BUTTON_NORMAL;
        }
    }
}