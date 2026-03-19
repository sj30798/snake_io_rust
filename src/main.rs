mod game;

use bevy::prelude::*;
use bevy::input::keyboard::KeyCode;
use bevy::window::WindowResolution;
use std::sync::{Arc, Mutex};
use game::types::{Difficulty, Direction};
use game::core::Game;

const CELL_SIZE: f32 = 25.0;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    Menu,
    Playing,
}

#[derive(Resource)]
struct GameData {
    game: Arc<Mutex<Option<Game>>>,
    selected_difficulty: Difficulty,
    last_update: f64,
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

#[derive(Component)]
struct SnakeSegment;

#[derive(Component)]
struct Fruit;

#[derive(Component)]
struct UIText;

#[derive(Component)]
struct BoardBackground;

#[derive(Component)]
struct MenuUI;

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
        .insert_resource(ClearColor(Color::srgb(0.05, 0.08, 0.12)))
        .insert_state(GameState::Menu)
        .init_resource::<GameData>()
        .add_systems(Startup, setup_camera)
        .add_systems(OnEnter(GameState::Menu), spawn_menu_ui)
        .add_systems(OnExit(GameState::Menu), cleanup_menu_ui)
        .add_systems(Update, menu_input.run_if(in_state(GameState::Menu)))
        .add_systems(OnEnter(GameState::Playing), start_game)
        .add_systems(Update, (
            game_input,
            game_update,
            render_game,
        ).run_if(in_state(GameState::Playing)))
        .add_systems(OnExit(GameState::Playing), cleanup_game)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_menu_ui(mut commands: Commands) {
    commands.spawn((
        TextBundle::from_section(
            "SNAKE IO\n\nPress 1: Easy\nPress 2: Normal\nPress 3: Hard\n\nMove: Arrow Keys\nQuit: Q or Esc",
            TextStyle {
                font_size: 34.0,
                color: Color::srgb(0.95, 0.95, 0.95),
                font: default(),
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            left: Val::Px(40.0),
            top: Val::Px(40.0),
            ..default()
        }),
        MenuUI,
    ));
}

fn cleanup_menu_ui(mut commands: Commands, menu: Query<Entity, With<MenuUI>>) {
    for entity in &menu {
        commands.entity(entity).despawn_recursive();
    }
}

fn menu_input(
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

fn start_game(mut game_data: ResMut<GameData>) {
    let settings = game_data.selected_difficulty.settings();
    if let Ok(mut game) = game_data.game.lock() {
        *game = Some(Game::new(settings));
    }
    game_data.last_update = 0.0;
}

fn game_input(
    mut game_data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if let Ok(mut game_opt) = game_data.game.lock() {
        if let Some(game) = game_opt.as_mut() {
            if keyboard.just_pressed(KeyCode::ArrowUp) {
                if let Some(player) = game.player_mut() {
                    player.set_direction(Direction::Up);
                }
            }
            if keyboard.just_pressed(KeyCode::ArrowDown) {
                if let Some(player) = game.player_mut() {
                    player.set_direction(Direction::Down);
                }
            }
            if keyboard.just_pressed(KeyCode::ArrowLeft) {
                if let Some(player) = game.player_mut() {
                    player.set_direction(Direction::Left);
                }
            }
            if keyboard.just_pressed(KeyCode::ArrowRight) {
                if let Some(player) = game.player_mut() {
                    player.set_direction(Direction::Right);
                }
            }
            if keyboard.just_pressed(KeyCode::KeyQ) || keyboard.just_pressed(KeyCode::Escape) {
                game.quit_requested = true;
            }
        }
    }
}

fn game_update(
    mut game_data: ResMut<GameData>,
    mut next_state: ResMut<NextState<GameState>>,
    time: Res<Time>,
) {
    let elapsed = time.elapsed_seconds_f64();
    let last_update = game_data.last_update;
    let mut advanced_tick = false;
    let mut finished_round = false;

    if let Ok(mut game_opt) = game_data.game.lock() {
        if let Some(game) = game_opt.as_mut() {
            let tick_duration: f64 = game.get_current_tick_duration().as_secs_f64();

            if elapsed - last_update >= tick_duration {
                game.update_bot_directions();
                game.move_snakes();
                game.resolve_head_to_head();
                game.resolve_wall_and_self_collisions();
                game.resolve_snake_eating();
                game.resolve_cross_snake_overlaps();
                game.resolve_fruit_eating();
                game.update_high_score();
                game.refill_fruits();

                advanced_tick = true;
                finished_round = game.get_is_finished();
            }
        }
    }

    if advanced_tick {
        game_data.last_update = elapsed;
    }

    if finished_round {
        next_state.set(GameState::Menu);
    }
}

fn render_game(
    game_data: Res<GameData>,
    mut commands: Commands,
    board_background: Query<Entity, With<BoardBackground>>,
    snake_segments: Query<Entity, With<SnakeSegment>>,
    fruits: Query<Entity, With<Fruit>>,
    ui_texts: Query<Entity, With<UIText>>,
) {
    for entity in &board_background {
        commands.entity(entity).despawn();
    }

    // Clear existing entities
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
            let game_width = game::types::WIDTH as f32;
            let game_height = game::types::HEIGHT as f32;
            let offset_x = -(game_width * CELL_SIZE) / 2.0;
            let offset_y = (game_height * CELL_SIZE) / 2.0;

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.12, 0.16, 0.20),
                        custom_size: Some(Vec2::new(game_width * CELL_SIZE, game_height * CELL_SIZE)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, -0.5)),
                    ..default()
                },
                BoardBackground,
            ));

            // Render fruits
            for fruit in &game.fruits {
                if game::types::in_bounds(*fruit) {
                    let x = offset_x + fruit.x as f32 * CELL_SIZE + CELL_SIZE / 2.0;
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

            // Render snakes
            for snake in game.snakes.iter().filter(|s| s.alive) {
                for (i, segment) in snake.body.iter().enumerate() {
                    if !game::types::in_bounds(*segment) {
                        continue;
                    }
                    
                    let x = offset_x + segment.x as f32 * CELL_SIZE + CELL_SIZE / 2.0;
                    let y = offset_y - segment.y as f32 * CELL_SIZE - CELL_SIZE / 2.0;
                    
                    let color = match snake.color {
                        crossterm::style::Color::Red => Color::srgb(1.0, 0.0, 0.0),
                        crossterm::style::Color::Green => Color::srgb(0.0, 1.0, 0.0),
                        crossterm::style::Color::Blue => Color::srgb(0.0, 0.0, 1.0),
                        crossterm::style::Color::Yellow => Color::srgb(1.0, 1.0, 0.0),
                        crossterm::style::Color::Magenta => Color::srgb(1.0, 0.0, 1.0),
                        crossterm::style::Color::Cyan => Color::srgb(0.0, 1.0, 1.0),
                        crossterm::style::Color::White => Color::srgb(1.0, 1.0, 1.0),
                        crossterm::style::Color::Grey => Color::srgb(0.5, 0.5, 0.5),
                        _ => Color::srgb(0.7, 0.7, 0.7),
                    };
                    
                    let size = if i == 0 { CELL_SIZE - 2.0 } else { CELL_SIZE - 3.0 };
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
                }
            }

            // Render UI text
            let player_score = game.player().map_or(0, |p: &_| p.len());
            let alive_snakes = game.snakes.iter().filter(|s| s.alive).count();
            
            commands.spawn((
                TextBundle::from_section(
                    format!(
                        "Snake IO [{}]  Score: {}  High: {}  Time: {}s  Snakes: {}",
                        game.settings.label(),
                        player_score,
                        game.high_score,
                        game.remaining_seconds(),
                        alive_snakes
                    ),
                    TextStyle {
                        font_size: 20.0,
                        color: Color::WHITE,
                        font: default(),
                    },
                )
                .with_style(Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(14.0),
                    top: Val::Px(10.0),
                    ..default()
                }),
                UIText,
            ));
            
            commands.spawn((
                TextBundle::from_section(
                    "Arrow keys: move | Q: quit",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::WHITE,
                        font: default(),
                    },
                )
                .with_style(Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(14.0),
                    top: Val::Px(36.0),
                    ..default()
                }),
                UIText,
            ));
        }
    }
}

fn cleanup_game(mut game_data: ResMut<GameData>) {
    if let Ok(mut game) = game_data.game.lock() {
        *game = None;
    }
}