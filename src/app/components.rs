//! ECS marker components used by UI and rendering systems.

use bevy::prelude::*;

/// Marker for snake segment sprites.
#[derive(Component)]
pub struct SnakeSegment;

/// Marker for fruit sprites.
#[derive(Component)]
pub struct Fruit;

/// Marker for gameplay UI entities.
#[derive(Component)]
pub struct UIText;

/// Marker for board background, frame, and grid decor entities.
#[derive(Component)]
pub struct BoardDecor;

/// Marker for menu overlay UI entities.
#[derive(Component)]
pub struct MenuUI;
