//! Binary entrypoint for Snake IO.

mod app;
mod game;

use bevy::prelude::*;
use bevy::window::WindowResolution;

use crate::app::SnakeIoAppPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(1200.0, 500.0),
                title: "Snake IO (Rust)".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(SnakeIoAppPlugin)
        .run();
}
