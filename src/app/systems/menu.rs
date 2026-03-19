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

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn format_last_results(results: Option<&LastRoundResults>) -> String {
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

/// Spawns translucent menu overlay and menu card.
pub fn spawn_menu_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_data: Res<GameData>,
) {
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

                        section.spawn(
                            TextBundle::from_section(
                                if game_data.first_run {
                                    "First run: opening round starts on Easy for onboarding"
                                } else {
                                    "Press 1/2/3 to start instantly"
                                },
                                TextStyle {
                                    font_size: 14.0,
                                    color: Color::srgb(0.74, 0.95, 0.87),
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
                            "[1] Easy     - 2 bots, slower speed\n[2] Normal  - 3 bots, moderate speed\n[3] Hard    - 5 bots, fast speed\n\nDaily challenge seed refreshes each run.",
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
                            "Arrow/WASD - Move\nShift - Sprint burst\nSpace - Magnet pulse\nM / C / V - Accessibility toggles\nQ or ESC - Quit round",
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
                            "Eat fruit to grow and score.\nChain quick pickups for combo bonuses.\nOrange zone fruit grants bonus growth.\nBe the last snake standing!",
                            TextStyle {
                                font_size: 16.0,
                                color: Color::srgb(0.9, 0.95, 1.0),
                                font: load_game_font(&asset_server),
                            },
                        ));
                    });

                    card.spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            width: Val::Percent(100.0),
                            padding: UiRect::all(Val::Px(16.0)),
                            row_gap: Val::Px(8.0),
                            ..default()
                        },
                        background_color: BackgroundColor(Color::srgba(0.22, 0.88, 0.70, 0.14)),
                        ..default()
                    })
                    .with_children(|section| {
                        section.spawn(TextBundle::from_section(
                            "SESSION PROGRESSION",
                            TextStyle {
                                font_size: 18.0,
                                color: Color::srgb(0.53, 1.0, 0.84),
                                font: load_game_font(&asset_server),
                            },
                        ));

                        section.spawn(TextBundle::from_section(
                            format!(
                                "Runs: {}\nQuest Clears: {}\nUnlocked Trails: {}\nDaily Best: {}\nTip: {}",
                                game_data.meta_progress.runs_completed,
                                game_data.meta_progress.total_quest_completions,
                                game_data.meta_progress.unlocked_trails,
                                game_data.meta_progress.daily_best_score,
                                game_data.contextual_tip,
                            ),
                            TextStyle {
                                font_size: 15.0,
                                color: Color::srgb(0.92, 1.0, 0.96),
                                font: load_game_font(&asset_server),
                            },
                        ));
                    });
                });

        });
}

/// Spawns post-round summary overlay with replay options.
pub fn spawn_round_summary_ui(
    mut commands: Commands,
    game_data: Res<GameData>,
    asset_server: Res<AssetServer>,
) {
    let (winner_title, winner_is_player, meta_text, rankings_text) = if let Some(results) = game_data.last_results.as_ref() {
        let winner = results
            .entries
            .first()
            .map(|entry| {
                let status = if entry.alive { "SURVIVED" } else { "OUT" };
                format!("Winner: {}  |  Score: {}  |  {}", entry.name, entry.score, status)
            })
            .unwrap_or_else(|| "Winner: N/A".to_string());
        let winner_is_player = results
            .entries
            .first()
            .is_some_and(|entry| entry.name == "You");

        let meta = format!(
            "Mode: {}\nHigh Score: {}\nTime Left: {}s\nBest Combo: x{}\nNear-Miss Bonus: {}\nQuest: {}",
            results.mode_label,
            results.high_score,
            results.time_left,
            results.best_combo,
            results.near_miss_bonus,
            if results.quest_completed { "Complete" } else { "In progress" },
        );

        let mut lines = vec!["Rankings".to_string(), "".to_string()];
        for row in &results.entries {
            let badge = match row.rank {
                1 => "[1st]",
                2 => "[2nd]",
                3 => "[3rd]",
                _ => "[   ]",
            };
            let status = if row.alive { "alive" } else { "out" };
            lines.push(format!("{} {} - {} ({})", badge, row.name, row.score, status));
        }

        (winner, winner_is_player, meta, lines.join("\n"))
    } else {
        (
            "Winner: N/A".to_string(),
            false,
            "No completed game found.".to_string(),
            "Rankings\n\nStart a round to see standings.".to_string(),
        )
    };

    let winner_bg = if winner_is_player {
        Color::srgba(0.08, 0.26, 0.18, 0.94)
    } else {
        Color::srgba(0.26, 0.14, 0.14, 0.94)
    };
    let winner_fg = if winner_is_player {
        Color::srgb(0.65, 1.0, 0.78)
    } else {
        Color::srgb(1.0, 0.78, 0.68)
    };

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
                background_color: BackgroundColor(Color::srgba(0.01, 0.03, 0.06, 0.72)),
                ..default()
            },
            MenuUI,
        ))
        .with_children(|root| {
            root.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(780.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(28.0)),
                    row_gap: Val::Px(16.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0.09, 0.13, 0.19, 0.95)),
                ..default()
            })
            .with_children(|card| {
                card.spawn(TextBundle::from_section(
                    "ROUND COMPLETE",
                    TextStyle {
                        font_size: 44.0,
                        color: Color::srgb(1.0, 0.88, 0.2),
                        font: load_game_font(&asset_server),
                    },
                ));

                card.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        padding: UiRect::all(Val::Px(14.0)),
                        ..default()
                    },
                    background_color: BackgroundColor(winner_bg),
                    ..default()
                })
                .with_children(|panel| {
                    panel.spawn(TextBundle::from_section(
                        winner_title,
                        TextStyle {
                            font_size: 22.0,
                            color: winner_fg,
                            font: load_game_font(&asset_server),
                        },
                    ));
                });

                card.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        padding: UiRect::all(Val::Px(14.0)),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgba(0.08, 0.16, 0.25, 0.90)),
                    ..default()
                })
                .with_children(|panel| {
                    panel.spawn(TextBundle::from_section(
                        meta_text,
                        TextStyle {
                            font_size: 18.0,
                            color: Color::srgb(0.88, 0.95, 1.0),
                            font: load_game_font(&asset_server),
                        },
                    ));
                });

                card.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        padding: UiRect::all(Val::Px(14.0)),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgba(0.11, 0.14, 0.22, 0.94)),
                    ..default()
                })
                .with_children(|panel| {
                    panel.spawn(TextBundle::from_section(
                        rankings_text,
                        TextStyle {
                            font_size: 17.0,
                            color: Color::srgb(0.92, 0.96, 1.0),
                            font: load_game_font(&asset_server),
                        },
                    ));
                });

                let death_tip = match game_data
                    .last_results
                    .as_ref()
                    .and_then(|r| r.player_death_cause)
                {
                    Some(crate::game::types::DeathCause::Wall) => "Tip: keep one tile of safety near walls.",
                    Some(crate::game::types::DeathCause::SelfCollision) => "Tip: avoid tight loops right after sprint.",
                    Some(crate::game::types::DeathCause::HeadToHead) => "Tip: only contest enemy heads when longer.",
                    Some(crate::game::types::DeathCause::EatenBySnake) => "Tip: avoid crossing enemy bodies near heads.",
                    Some(crate::game::types::DeathCause::TimeUp) => "Tip: prioritize combo chains for faster score growth.",
                    Some(crate::game::types::DeathCause::Quit) => "Tip: instant replay with R keeps your rhythm.",
                    _ => "Tip: hold center control and farm clustered fruit.",
                };

                card.spawn(TextBundle::from_section(
                    format!(
                        "Press Enter/Space: Back to menu\nPress R: Instant replay\nPress 1/2/3: Play again (Easy/Normal/Hard)\n\n{}",
                        death_tip
                    ),
                    TextStyle {
                        font_size: 18.0,
                        color: Color::srgb(0.67, 0.93, 1.0),
                        font: load_game_font(&asset_server),
                    },
                ));
            });
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

/// Handles post-round summary hotkeys.
pub fn summary_input(
    mut next_state: ResMut<NextState<GameState>>,
    mut game_data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Menu);
    } else if keyboard.just_pressed(KeyCode::KeyR) {
        next_state.set(GameState::Playing);
    } else if keyboard.just_pressed(KeyCode::Digit1) {
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
