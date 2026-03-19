use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use crate::app::components::{BoardDecor, Fruit, MenuUI, SnakeSegment, UIText};
use crate::app::state::{GameData, GameState};
use crate::app::systems::gameplay::{cleanup_game, game_input, game_update, start_game};
use crate::app::systems::menu::{cleanup_menu_ui, menu_input};
use crate::game::core::Game;
use crate::game::types::{Difficulty, Direction};

#[test]
fn menu_input_transitions_to_playing_and_sets_difficulty() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_state(GameState::Menu)
        .init_resource::<GameData>()
        .insert_resource(ButtonInput::<KeyCode>::default())
        .add_systems(Update, menu_input.run_if(in_state(GameState::Menu)));

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Digit1);

    // One frame to set NextState, one frame to apply transition.
    app.update();
    app.update();

    let state = app.world().resource::<State<GameState>>();
    assert!(matches!(state.get(), GameState::Playing));
    let data = app.world().resource::<GameData>();
    assert!(matches!(data.selected_difficulty, Difficulty::Easy));
}

#[test]
fn start_game_creates_game_instance() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_resource::<GameData>()
        .add_systems(Update, start_game);

    app.world_mut()
        .resource_mut::<GameData>()
        .selected_difficulty = Difficulty::Hard;
    app.update();

    let data = app.world().resource::<GameData>();
    let game_guard = data.game.lock().expect("game mutex should lock");
    let game = game_guard.as_ref().expect("game should be initialized");
    assert!(matches!(game.settings.difficulty, Difficulty::Hard));
}

#[test]
fn game_update_transitions_back_to_menu_when_finished() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_state(GameState::Playing)
        .init_resource::<GameData>()
        .insert_resource(Time::<()>::default())
        .add_systems(Update, game_update.run_if(in_state(GameState::Playing)));

    let game_handle = {
        let mut data = app.world_mut().resource_mut::<GameData>();
        data.last_update = -1.0;
        data.game.clone()
    };
    let mut game = Game::new(Difficulty::Easy.settings());
    game.quit_requested = true;
    if let Ok(mut guard) = game_handle.lock() {
        *guard = Some(game);
    };

    // One frame to set NextState, one frame to apply transition.
    app.update();
    app.update();

    let state = app.world().resource::<State<GameState>>();
    assert!(matches!(state.get(), GameState::Menu));
}

#[test]
fn cleanup_menu_ui_despawns_menu_entities() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, cleanup_menu_ui);

    app.world_mut().spawn(MenuUI);
    app.world_mut().spawn((MenuUI, Name::new("menu-root")));

    app.update();

    let remaining = app
        .world()
        .iter_entities()
        .filter(|entity| entity.contains::<MenuUI>())
        .count();
    assert_eq!(remaining, 0);
}

#[test]
fn cleanup_game_clears_model_and_gameplay_entities() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<GameData>()
        .add_systems(Update, cleanup_game);

    {
        let data = app.world().resource::<GameData>();
        let mut guard = data.game.lock().expect("game mutex should lock");
        *guard = Some(Game::new(Difficulty::Normal.settings()));
    }

    app.world_mut().spawn(BoardDecor);
    app.world_mut().spawn(SnakeSegment);
    app.world_mut().spawn(Fruit);
    app.world_mut().spawn(UIText);

    app.update();

    let data = app.world().resource::<GameData>();
    let guard = data.game.lock().expect("game mutex should lock");
    assert!(guard.is_none());

    let board_count = app
        .world()
        .iter_entities()
        .filter(|entity| entity.contains::<BoardDecor>())
        .count();
    let snake_count = app
        .world()
        .iter_entities()
        .filter(|entity| entity.contains::<SnakeSegment>())
        .count();
    let fruit_count = app
        .world()
        .iter_entities()
        .filter(|entity| entity.contains::<Fruit>())
        .count();
    let ui_count = app
        .world()
        .iter_entities()
        .filter(|entity| entity.contains::<UIText>())
        .count();

    assert_eq!(board_count, 0);
    assert_eq!(snake_count, 0);
    assert_eq!(fruit_count, 0);
    assert_eq!(ui_count, 0);
}

#[test]
fn game_input_updates_player_direction_on_arrow_key() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<GameData>()
        .insert_resource(ButtonInput::<KeyCode>::default())
        .add_systems(Update, game_input);

    {
        let data = app.world().resource::<GameData>();
        let mut guard = data.game.lock().expect("game mutex should lock");
        *guard = Some(Game::new(Difficulty::Easy.settings()));
    }

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ArrowUp);
    app.update();

    let data = app.world().resource::<GameData>();
    let guard = data.game.lock().expect("game mutex should lock");
    let player = guard
        .as_ref()
        .and_then(|game| game.player())
        .expect("player should exist");
    assert!(matches!(player.next_dir, Direction::Up));
}

#[test]
fn game_input_rejects_immediate_reverse_turn() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<GameData>()
        .insert_resource(ButtonInput::<KeyCode>::default())
        .add_systems(Update, game_input);

    {
        let data = app.world().resource::<GameData>();
        let mut guard = data.game.lock().expect("game mutex should lock");
        *guard = Some(Game::new(Difficulty::Easy.settings()));
    }

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ArrowLeft);
    app.update();

    let data = app.world().resource::<GameData>();
    let guard = data.game.lock().expect("game mutex should lock");
    let player = guard
        .as_ref()
        .and_then(|game| game.player())
        .expect("player should exist");
    assert!(matches!(player.dir, Direction::Right));
    assert!(matches!(player.next_dir, Direction::Right));
}

#[test]
fn game_input_sets_quit_requested_on_escape() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<GameData>()
        .insert_resource(ButtonInput::<KeyCode>::default())
        .add_systems(Update, game_input);

    {
        let data = app.world().resource::<GameData>();
        let mut guard = data.game.lock().expect("game mutex should lock");
        *guard = Some(Game::new(Difficulty::Easy.settings()));
    }

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Escape);
    app.update();

    let data = app.world().resource::<GameData>();
    let guard = data.game.lock().expect("game mutex should lock");
    let game = guard.as_ref().expect("game should exist");
    assert!(game.quit_requested);
}

#[test]
fn game_input_is_noop_when_game_missing() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<GameData>()
        .insert_resource(ButtonInput::<KeyCode>::default())
        .add_systems(Update, game_input);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ArrowUp);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Escape);
    app.update();

    let data = app.world().resource::<GameData>();
    let guard = data.game.lock().expect("game mutex should lock");
    assert!(guard.is_none());
}
