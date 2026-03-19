//! Rendering systems for board, entities, and in-game HUD.

use bevy::prelude::*;
use std::path::PathBuf;

use crate::app::components::{BoardDecor, Fruit, SnakeSegment, UIText};
use crate::app::state::{GameData, CELL_SIZE, SIDE_PANEL_WIDTH};
use crate::game::types::{EyeStyle, PatternType, RGB};

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

/// Converts RGB to Bevy Color
fn rgb_to_bevy_color(rgb: RGB) -> Color {
    Color::srgb(rgb.r, rgb.g, rgb.b)
}

/// Calculates segment color based on skin pattern and position
fn calculate_segment_color(
    rgb: RGB,
    pattern_rgb: RGB,
    pattern_type: PatternType,
    segment_index: usize,
    is_head: bool,
) -> Color {
    let color = match pattern_type {
        PatternType::Solid => rgb,
        PatternType::Stripes => {
            if segment_index % 2 == 0 {
                rgb
            } else {
                pattern_rgb
            }
        }
        PatternType::Dots => {
            if segment_index % 3 == 0 {
                pattern_rgb
            } else {
                rgb
            }
        }
        PatternType::Gradient => {
            let fade = (segment_index as f32) * 0.08;
            let fade = fade.min(0.4);
            RGB::new(
                (rgb.r * (1.0 - fade)).max(0.0),
                (rgb.g * (1.0 - fade)).max(0.0),
                (rgb.b * (1.0 - fade)).max(0.0),
            )
        }
        PatternType::Scales => {
            let variation = if segment_index % 2 == 0 { 0.95 } else { 1.05 };
            RGB::new(
                (rgb.r * variation).min(1.0),
                (rgb.g * variation).min(1.0),
                (rgb.b * variation).min(1.0),
            )
        }
    };

    // Head is always brighter
    let final_color = if is_head {
        RGB::new(
            (color.r * 1.2).min(1.0),
            (color.g * 1.2).min(1.0),
            (color.b * 1.2).min(1.0),
        )
    } else {
        color
    };

    rgb_to_bevy_color(final_color)
}

/// Renders board decor, fruits, snakes, and side panel HUD from current game state.
pub fn render_game(
    game_data: Res<GameData>,
    mut commands: Commands,
    board_decor: Query<Entity, With<BoardDecor>>,
    snake_segments: Query<Entity, With<SnakeSegment>>,
    fruits: Query<Entity, With<Fruit>>,
    ui_texts: Query<Entity, With<UIText>>,
    asset_server: Res<AssetServer>,
) {
    for entity in &board_decor {
        commands.entity(entity).despawn();
    }

    for entity in &snake_segments {
        commands.entity(entity).despawn();
    }
    for entity in &fruits {
        commands.entity(entity).despawn();
    }
    for entity in &ui_texts {
        commands.entity(entity).despawn();
    }

    if let Ok(game_opt) = game_data.game.lock() {
        if let Some(game) = game_opt.as_ref() {
            let game_width = crate::game::types::WIDTH as f32;
            let game_height = crate::game::types::HEIGHT as f32;
            let offset_x = -(game_width * CELL_SIZE) / 2.0;
            let offset_y = (game_height * CELL_SIZE) / 2.0;
            let board_x_shift = -SIDE_PANEL_WIDTH * 0.45;

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.11, 0.15, 0.19),
                        custom_size: Some(Vec2::new(
                            game_width * CELL_SIZE,
                            game_height * CELL_SIZE,
                        )),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(board_x_shift, 0.0, -0.5)),
                    ..default()
                },
                BoardDecor,
            ));

            let board_w = game_width * CELL_SIZE;
            let board_h = game_height * CELL_SIZE;
            let border_color = Color::srgb(0.34, 0.54, 0.70);
            let border_thickness = 2.0;
            let half_w = board_w / 2.0;
            let half_h = board_h / 2.0;
            let bx = board_x_shift;

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: border_color,
                        custom_size: Some(Vec2::new(
                            board_w + border_thickness * 2.0,
                            border_thickness,
                        )),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(
                        bx,
                        half_h + border_thickness / 2.0,
                        -0.2,
                    )),
                    ..default()
                },
                BoardDecor,
            ));
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: border_color,
                        custom_size: Some(Vec2::new(
                            board_w + border_thickness * 2.0,
                            border_thickness,
                        )),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(
                        bx,
                        -half_h - border_thickness / 2.0,
                        -0.2,
                    )),
                    ..default()
                },
                BoardDecor,
            ));
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: border_color,
                        custom_size: Some(Vec2::new(
                            border_thickness,
                            board_h + border_thickness * 2.0,
                        )),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(
                        bx - half_w - border_thickness / 2.0,
                        0.0,
                        -0.2,
                    )),
                    ..default()
                },
                BoardDecor,
            ));
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: border_color,
                        custom_size: Some(Vec2::new(
                            border_thickness,
                            board_h + border_thickness * 2.0,
                        )),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(
                        bx + half_w + border_thickness / 2.0,
                        0.0,
                        -0.2,
                    )),
                    ..default()
                },
                BoardDecor,
            ));

            for gx in 1..(crate::game::types::WIDTH as usize) {
                let x = offset_x + gx as f32 * CELL_SIZE + board_x_shift;
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::srgba(0.70, 0.82, 0.92, 0.07),
                            custom_size: Some(Vec2::new(1.0, board_h)),
                            ..default()
                        },
                        transform: Transform::from_translation(Vec3::new(x, 0.0, -0.3)),
                        ..default()
                    },
                    BoardDecor,
                ));
            }

            for gy in 1..(crate::game::types::HEIGHT as usize) {
                let y = offset_y - gy as f32 * CELL_SIZE;
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::srgba(0.70, 0.82, 0.92, 0.07),
                            custom_size: Some(Vec2::new(board_w, 1.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(Vec3::new(board_x_shift, y, -0.3)),
                        ..default()
                    },
                    BoardDecor,
                ));
            }

            for fruit in &game.fruits {
                if crate::game::types::in_bounds(*fruit) {
                    let x = offset_x + fruit.x as f32 * CELL_SIZE + CELL_SIZE / 2.0 + board_x_shift;
                    let y = offset_y - fruit.y as f32 * CELL_SIZE - CELL_SIZE / 2.0;
                    commands.spawn((
                        SpriteBundle {
                            sprite: Sprite {
                                color: Color::srgb(1.0, 1.0, 0.0),
                                custom_size: Some(Vec2::new(CELL_SIZE - 2.0, CELL_SIZE - 2.0)),
                                ..default()
                            },
                            transform: Transform::from_translation(Vec3::new(x, y, 0.0)),
                            ..default()
                        },
                        Fruit,
                    ));
                }
            }

            for snake in game.snakes.iter().filter(|s| s.alive) {
                for (i, segment) in snake.body.iter().enumerate() {
                    if !crate::game::types::in_bounds(*segment) {
                        continue;
                    }

                    let x =
                        offset_x + segment.x as f32 * CELL_SIZE + CELL_SIZE / 2.0 + board_x_shift;
                    let y = offset_y - segment.y as f32 * CELL_SIZE - CELL_SIZE / 2.0;

                    let is_head = i == 0;
                    let color = calculate_segment_color(
                        snake.skin.base_color,
                        snake.skin.pattern_color,
                        snake.skin.pattern_type,
                        i,
                        is_head,
                    );

                    let size = if is_head {
                        CELL_SIZE - 2.0
                    } else {
                        CELL_SIZE - 3.0
                    };

                    commands.spawn((
                        SpriteBundle {
                            sprite: Sprite {
                                color,
                                custom_size: Some(Vec2::new(size, size)),
                                ..default()
                            },
                            transform: Transform::from_translation(Vec3::new(x, y, 1.0)),
                            ..default()
                        },
                        SnakeSegment,
                    ));

                    // Draw eyes on head
                    if is_head && snake.skin.eye_style != EyeStyle::None {
                        let eye_color = Color::srgb(0.0, 0.0, 0.0);
                        let eye_size = CELL_SIZE * 0.15;
                        let eye_offset = CELL_SIZE * 0.22;

                        match snake.skin.eye_style {
                            EyeStyle::Round => {
                                // Two round eyes
                                for eye_x_offset in [-eye_offset, eye_offset] {
                                    commands.spawn((
                                        SpriteBundle {
                                            sprite: Sprite {
                                                color: eye_color,
                                                custom_size: Some(Vec2::new(eye_size, eye_size)),
                                                ..default()
                                            },
                                            transform: Transform::from_translation(Vec3::new(
                                                x + eye_x_offset,
                                                y + CELL_SIZE * 0.15,
                                                2.0,
                                            )),
                                            ..default()
                                        },
                                        SnakeSegment,
                                    ));
                                }
                            }
                            EyeStyle::Oval => {
                                // Two oval eyes (wider)
                                for eye_x_offset in [-eye_offset, eye_offset] {
                                    commands.spawn((
                                        SpriteBundle {
                                            sprite: Sprite {
                                                color: eye_color,
                                                custom_size: Some(Vec2::new(
                                                    eye_size * 1.5,
                                                    eye_size * 0.8,
                                                )),
                                                ..default()
                                            },
                                            transform: Transform::from_translation(Vec3::new(
                                                x + eye_x_offset,
                                                y + CELL_SIZE * 0.15,
                                                2.0,
                                            )),
                                            ..default()
                                        },
                                        SnakeSegment,
                                    ));
                                }
                            }
                            EyeStyle::Angular => {
                                // Two angular eyes (narrower)
                                for eye_x_offset in [-eye_offset, eye_offset] {
                                    commands.spawn((
                                        SpriteBundle {
                                            sprite: Sprite {
                                                color: eye_color,
                                                custom_size: Some(Vec2::new(
                                                    eye_size * 0.7,
                                                    eye_size * 1.3,
                                                )),
                                                ..default()
                                            },
                                            transform: Transform::from_translation(Vec3::new(
                                                x + eye_x_offset,
                                                y + CELL_SIZE * 0.15,
                                                2.0,
                                            )),
                                            ..default()
                                        },
                                        SnakeSegment,
                                    ));
                                }
                            }
                            EyeStyle::None => {}
                        }
                    }
                }
            }

            let player_score = game.player().map_or(0, |p| p.len());
            let alive_snakes = game.snakes.iter().filter(|s| s.alive).count();

            commands.spawn((
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        right: Val::Px(0.0),
                        top: Val::Px(0.0),
                        width: Val::Px(SIDE_PANEL_WIDTH),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgba(0.03, 0.06, 0.09, 0.94)),
                    ..default()
                },
                UIText,
            ));

            commands.spawn((
                TextBundle::from_section(
                    format!(
                        "SNAKE IO\n\nMode: {}\nScore: {}\nHigh Score: {}\nTime Left: {}s\nAlive: {}",
                        game.settings.label(),
                        player_score,
                        game.high_score,
                        game.remaining_seconds(),
                        alive_snakes
                    ),
                    TextStyle {
                        font_size: 20.0,
                        color: Color::WHITE,
                        font: load_game_font(&asset_server),
                    },
                )
                .with_style(Style {
                    position_type: PositionType::Absolute,
                    right: Val::Px(26.0),
                    top: Val::Px(26.0),
                    ..default()
                }),
                UIText,
            ));

            commands.spawn((
                TextBundle::from_section(
                    "Controls\nArrow keys: Move\nQ / Esc: Exit round",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::srgb(0.72, 0.86, 1.0),
                        font: load_game_font(&asset_server),
                    },
                )
                .with_style(Style {
                    position_type: PositionType::Absolute,
                    right: Val::Px(26.0),
                    bottom: Val::Px(28.0),
                    ..default()
                }),
                UIText,
            ));
        }
    }
}
