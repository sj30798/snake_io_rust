//! Shared game state and constants for the Bevy app.

use std::sync::{Arc, Mutex};

use bevy::prelude::*;

use crate::game::core::Game;
use crate::game::types::Difficulty;

/// Visual size of a single board cell in world-space pixels.
pub const CELL_SIZE: f32 = 14.0;

/// Width of the right-side HUD panel in UI pixels.
pub const SIDE_PANEL_WIDTH: f32 = 290.0;

/// Top-level state machine for menu vs gameplay.
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    /// Menu and difficulty selection screen.
    #[default]
    Menu,
    /// Active gameplay loop.
    Playing,
}

/// Runtime resource containing active game model and player configuration.
#[derive(Resource)]
pub struct GameData {
    /// Current game instance, created when entering `Playing`.
    pub game: Arc<Mutex<Option<Game>>>,
    /// Difficulty selected in menu.
    pub selected_difficulty: Difficulty,
    /// Last simulation timestamp used for fixed-step updates.
    pub last_update: f64,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            game: Arc::new(Mutex::new(None)),
            selected_difficulty: Difficulty::Normal,
            last_update: 0.0,
        }
    }
}
