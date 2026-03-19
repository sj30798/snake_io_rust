//! Bevy app composition and system scheduling.

pub mod components;
pub mod state;
pub mod systems;

use bevy::prelude::*;

use crate::app::state::{GameData, GameState};
use crate::app::systems::gameplay::{cleanup_game, game_input, game_update, start_game};
use crate::app::systems::menu::{cleanup_menu_ui, menu_input, setup_camera, spawn_menu_ui};
use crate::app::systems::render::render_game;

/// Plugin that wires together game state, resources, and ECS systems.
pub struct SnakeIoAppPlugin;

impl Plugin for SnakeIoAppPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.05, 0.08, 0.12)))
            .insert_state(GameState::Menu)
            .init_resource::<GameData>()
            .add_systems(Startup, setup_camera)
            .add_systems(OnEnter(GameState::Menu), spawn_menu_ui)
            .add_systems(OnExit(GameState::Menu), cleanup_menu_ui)
            .add_systems(Update, menu_input.run_if(in_state(GameState::Menu)))
            .add_systems(OnEnter(GameState::Playing), start_game)
            .add_systems(
                Update,
                (game_input, game_update, render_game).run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_game);
    }
}

#[cfg(test)]
mod tests;
