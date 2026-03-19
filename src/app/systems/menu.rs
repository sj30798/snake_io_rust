//! Menu and camera systems.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use crate::app::components::MenuUI;
use crate::app::state::{GameData, GameState};
use crate::game::types::Difficulty;

/// Spawns the primary 2D camera.
pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

/// Spawns translucent menu overlay and menu card.
pub fn spawn_menu_ui(mut commands: Commands) {
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
                        width: Val::Px(540.0),
                        padding: UiRect::all(Val::Px(24.0)),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgba(0.07, 0.11, 0.16, 0.86)),
                    ..default()
                })
                .with_children(|card| {
                    card.spawn(TextBundle::from_section(
                        "SNAKE IO\n\nPress 1: Easy\nPress 2: Normal\nPress 3: Hard\n\nMove: Arrow Keys\nQuit: Q or Esc",
                        TextStyle {
                            font_size: 34.0,
                            color: Color::srgb(0.95, 0.95, 0.95),
                            font: default(),
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
