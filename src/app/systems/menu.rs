//! Menu and camera systems.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use std::path::PathBuf;

use crate::app::components::MenuUI;
use crate::app::state::{GameData, GameState, LastRoundResults};
use crate::game::types::Difficulty;

// Font asset path for custom game font
const GAME_FONT: &str = "fonts/FiraSans-Bold.ttf";

/// Loads the game font if it exists, otherwise returns default font
fn load_game_font(asset_server: &AssetServer) -> Handle<Font> {
    let mut font_path = PathBuf::from("assets");
    font_path.push("fonts");
    font_path.push("FiraSans-Bold.ttf");
    
    if font_path.exists() {
        asset_server.load(GAME_FONT)
    } else {
        Default::default()  // Falls back to Bevy's default font
    }
}

fn format_last_results(results: Option<&LastRoundResults>) -> String {
    if let Some(results) = results {
        let mut lines = vec![
            "Last Round".to_string(),
            String::new(),
            format!("Mode: {}", results.mode_label),
            format!("High Score: {}", results.high_score),
            format!("Time Left: {}s", results.time_left),
            String::new(),
            "Rankings".to_string(),
        ];

        for row in &results.entries {
            let status = if row.alive { "alive" } else { "out" };
            lines.push(format!("{}. {} - {} ({})", row.rank, row.name, row.score, status));
        }

        lines.join("\n")
    } else {
        "Last Round\n\nNo completed game yet.\nStart a round to see\nrankings and scores.".to_string()
    }
}

/// Spawns the primary 2D camera.
pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

/// Spawns translucent menu overlay, menu card, and conditionally shows last-round results panel.
pub fn spawn_menu_ui(mut commands: Commands, game_data: Res<GameData>, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0.02, 0.04, 0.07, 0.58)),
                ..default()
            },
            MenuUI,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(600.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(32.0)),
                        row_gap: Val::Px(24.0),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgba(0.08, 0.12, 0.18, 0.92)),
                    ..default()
                })
                .with_children(|card| {
                    // Title section
                    card.spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            width: Val::Percent(100.0),
                            padding: UiRect::bottom(Val::Px(16.0)),
                            row_gap: Val::Px(8.0),
                            ..default()
                        },
                        background_color: BackgroundColor(Color::srgba(0.0, 1.0, 1.0, 0.08)),
                        ..default()
                    })
                    .with_children(|section| {
                        section.spawn(
                            TextBundle::from_section(
                                ">> SNAKE IO <<",
                                TextStyle {
                                    font_size: 48.0,
                                    color: Color::srgb(0.0, 1.0, 1.0),
                                    font: load_game_font(&asset_server),
                                },
                            )
                            .with_style(Style {
                                justify_self: JustifySelf::Center,
                                ..default()
                            }),
                        );

                        section.spawn(
                            TextBundle::from_section(
                                "Multiplayer Snake Game",
                                TextStyle {
                                    font_size: 20.0,
                                    color: Color::srgb(0.6, 0.9, 1.0),
                                    font: load_game_font(&asset_server),
                                },
                            )
                            .with_style(Style {
                                justify_self: JustifySelf::Center,
                                ..default()
                            }),
                        );
                    });

                    // Difficulty section
                    card.spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            width: Val::Percent(100.0),
                            padding: UiRect::all(Val::Px(16.0)),
                            row_gap: Val::Px(10.0),
                            ..default()
                        },
                        background_color: BackgroundColor(Color::srgba(1.0, 0.4, 0.0, 0.2)),
                        ..default()
                    })
                    .with_children(|section| {
                        section.spawn(
                            TextBundle::from_section(
                                "SELECT DIFFICULTY",
                                TextStyle {
                                    font_size: 18.0,
                                    color: Color::srgb(1.0, 0.7, 0.0),
                                    font: load_game_font(&asset_server),
                                },
                            )
                            .with_style(Style {
                                margin: UiRect::bottom(Val::Px(4.0)),
                                ..default()
                            }),
                        );

                        section.spawn(TextBundle::from_section(
                            "[1] Easy     —  2 bots, slower speed\n[2] Normal  —  3 bots, moderate speed\n[3] Hard    —  5 bots, fast speed",
                            TextStyle {
                                font_size: 16.0,
                                color: Color::srgb(1.0, 1.0, 1.0),
                                font: load_game_font(&asset_server),
                            },
                        ));
                    });

                    // Controls section
                    card.spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            width: Val::Percent(100.0),
                            padding: UiRect::all(Val::Px(16.0)),
                            row_gap: Val::Px(10.0),
                            ..default()
                        },
                        background_color: BackgroundColor(Color::srgba(0.0, 1.0, 0.6, 0.15)),
                        ..default()
                    })
                    .with_children(|section| {
                        section.spawn(
                            TextBundle::from_section(
                                "GAMEPLAY CONTROLS",
                                TextStyle {
                                    font_size: 18.0,
                                    color: Color::srgb(0.0, 1.0, 0.7),
                                    font: load_game_font(&asset_server),
                                },
                            )
                            .with_style(Style {
                                margin: UiRect::bottom(Val::Px(4.0)),
                                ..default()
                            }),
                        );

                        section.spawn(TextBundle::from_section(
                            "Arrow Keys  —  Move your snake\nQ or ESC  —  Quit game",
                            TextStyle {
                                font_size: 16.0,
                                color: Color::srgb(0.9, 0.95, 1.0),
                                font: load_game_font(&asset_server),
                            },
                        ));
                    });

                    // Objective section
                    card.spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            width: Val::Percent(100.0),
                            padding: UiRect::all(Val::Px(16.0)),
                            row_gap: Val::Px(8.0),
                            ..default()
                        },
                        background_color: BackgroundColor(Color::srgba(1.0, 0.8, 0.0, 0.15)),
                        ..default()
                    })
                    .with_children(|section| {
                        section.spawn(
                            TextBundle::from_section(
                                "OBJECTIVE",
                                TextStyle {
                                    font_size: 18.0,
                                    color: Color::srgb(1.0, 0.9, 0.0),
                                    font: load_game_font(&asset_server),
                                },
                            )
                            .with_style(Style {
                                margin: UiRect::bottom(Val::Px(4.0)),
                                ..default()
                            }),
                        );

                        section.spawn(TextBundle::from_section(
                            "Eat fruit to grow longer and gain points.\nAvoid walls and other snakes.\nBe the last snake standing!",
                            TextStyle {
                                font_size: 16.0,
                                color: Color::srgb(0.9, 0.95, 1.0),
                                font: load_game_font(&asset_server),
                            },
                        ));
                    });
                });

            if game_data.last_results.is_some() {
                let results_text = format_last_results(game_data.last_results.as_ref());
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            right: Val::Px(26.0),
                            bottom: Val::Px(26.0),
                            width: Val::Px(360.0),
                            padding: UiRect::all(Val::Px(20.0)),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(12.0),
                            ..default()
                        },
                        background_color: BackgroundColor(Color::srgba(0.1, 0.15, 0.22, 0.95)),
                        ..default()
                    })
                    .with_children(|panel| {
                        panel.spawn(
                            TextBundle::from_section(
                                "★ LAST GAME RESULTS ★",
                                TextStyle {
                                    font_size: 18.0,
                                    color: Color::srgb(1.0, 0.9, 0.0),
                                    font: load_game_font(&asset_server),
                                },
                            )
                            .with_style(Style {
                                justify_self: JustifySelf::Center,
                                margin: UiRect::bottom(Val::Px(8.0)),
                                ..default()
                            }),
                        );

                        panel.spawn(TextBundle::from_section(
                            results_text,
                            TextStyle {
                                font_size: 16.0,
                                color: Color::srgb(0.85, 0.95, 1.0),
                                font: load_game_font(&asset_server),
                            },
                        ));
                    });
            }
        });
}

/// Removes menu overlay entities when transitioning into gameplay.
pub fn cleanup_menu_ui(mut commands: Commands, menu: Query<Entity, With<MenuUI>>) {
    for entity in &menu {
        commands.entity(entity).despawn_recursive();
    }
}

/// Handles menu difficulty selection hotkeys.
pub fn menu_input(
    mut next_state: ResMut<NextState<GameState>>,
    mut game_data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Digit1) {
        game_data.selected_difficulty = Difficulty::Easy;
        next_state.set(GameState::Playing);
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        game_data.selected_difficulty = Difficulty::Normal;
        next_state.set(GameState::Playing);
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        game_data.selected_difficulty = Difficulty::Hard;
        next_state.set(GameState::Playing);
    }
}
