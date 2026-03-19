use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::asset::AssetPlugin;

use crate::app::components::{BoardDecor, Fruit, MenuUI, SnakeSegment, UIText};
use crate::app::state::{GameData, GameState, LastRoundResults, RoundRankEntry};
use crate::app::systems::gameplay::{cleanup_game, game_input, game_update, start_game};
use crate::app::systems::menu::{
    cleanup_menu_ui, format_last_results, menu_input, spawn_menu_ui, summary_input,
};
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
    assert!(matches!(state.get(), GameState::RoundSummary));
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
    drop(guard);

    let last_results = data
        .last_results
        .as_ref()
        .expect("cleanup should snapshot last-round results");
    assert!(!last_results.entries.is_empty());
    assert_eq!(last_results.entries[0].rank, 1);
    assert!(last_results.entries.iter().any(|row| row.name == "You"));

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

#[test]
fn spawn_menu_ui_contains_difficulty_text() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_resource::<GameData>()
        .add_systems(Update, spawn_menu_ui);

    app.update();

    let mut found_title = false;
    let mut found_options = false;

    for entity in app.world().iter_entities() {
        if let Some(text) = entity.get::<Text>() {
            for section in &text.sections {
                let value = section.value.as_str();
                if value.contains("SELECT DIFFICULTY") {
                    found_title = true;
                }
                if value.contains("[1] Easy") && value.contains("[3] Hard") {
                    found_options = true;
                }
            }
        }
    }

    assert!(found_title, "menu should show SELECT DIFFICULTY heading");
    assert!(
        found_options,
        "menu should show all numbered difficulty options"
    );
}

#[test]
fn spawn_menu_ui_does_not_show_results_panel() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_resource::<GameData>()
        .add_systems(Update, spawn_menu_ui);

    // No results yet: panel title should not be present.
    app.update();
    let mut saw_results_title_without_data = false;
    for entity in app.world().iter_entities() {
        if let Some(text) = entity.get::<Text>() {
            if text
                .sections
                .iter()
                .any(|s| s.value.contains("LAST GAME RESULTS"))
            {
                saw_results_title_without_data = true;
                break;
            }
        }
    }
    assert!(
        !saw_results_title_without_data,
        "main menu should not render results panel before any round"
    );

    // Add a snapshot and run the system again.
    app.world_mut().resource_mut::<GameData>().last_results = Some(LastRoundResults {
        mode_label: "Hard".to_string(),
        high_score: 12,
        time_left: 7,
        entries: vec![RoundRankEntry {
            rank: 1,
            name: "You".to_string(),
            score: 12,
            alive: true,
        }],
        player_death_cause: None,
        best_combo: 3,
        near_miss_bonus: 1,
        quest_completed: false,
    });

    app.update();

    let mut saw_results_title_with_data = false;
    for entity in app.world().iter_entities() {
        if let Some(text) = entity.get::<Text>() {
            if text
                .sections
                .iter()
                .any(|s| s.value.contains("LAST GAME RESULTS"))
            {
                saw_results_title_with_data = true;
                break;
            }
        }
    }

    assert!(
        !saw_results_title_with_data,
        "main menu should not render results panel even when last_results exists"
    );
}

#[test]
fn spawn_menu_ui_contains_controls_and_objective_text() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_resource::<GameData>()
        .add_systems(Update, spawn_menu_ui);

    app.update();

    let mut found_controls_heading = false;
    let mut found_controls_details = false;
    let mut found_objective_heading = false;
    let mut found_objective_details = false;

    for entity in app.world().iter_entities() {
        if let Some(text) = entity.get::<Text>() {
            for section in &text.sections {
                let value = section.value.as_str();
                if value.contains("GAMEPLAY CONTROLS") {
                    found_controls_heading = true;
                }
                if value.contains("Arrow/WASD") && value.contains("Q or ESC") {
                    found_controls_details = true;
                }
                if value.contains("OBJECTIVE") {
                    found_objective_heading = true;
                }
                if value.contains("Eat fruit to grow")
                    && value.contains("Be the last snake standing")
                {
                    found_objective_details = true;
                }
            }
        }
    }

    assert!(
        found_controls_heading && found_controls_details,
        "menu should show controls heading and instructions"
    );
    assert!(
        found_objective_heading && found_objective_details,
        "menu should show objective heading and objective details"
    );
}

#[test]
fn format_last_results_renders_placeholder_and_rankings() {
    let empty = format_last_results(None);
    assert!(empty.contains("No completed game yet"));
    assert!(empty.contains("Start a round to see"));

    let data = LastRoundResults {
        mode_label: "Hard".to_string(),
        high_score: 21,
        time_left: 9,
        entries: vec![
            RoundRankEntry {
                rank: 1,
                name: "You".to_string(),
                score: 21,
                alive: true,
            },
            RoundRankEntry {
                rank: 2,
                name: "Bot 1".to_string(),
                score: 17,
                alive: false,
            },
        ],
        player_death_cause: None,
        best_combo: 5,
        near_miss_bonus: 2,
        quest_completed: true,
    };

    let rendered = format_last_results(Some(&data));
    assert!(rendered.contains("Mode: Hard"));
    assert!(rendered.contains("High Score: 21"));
    assert!(rendered.contains("Time Left: 9s"));
    assert!(rendered.contains("1. You - 21 (alive)"));
    assert!(rendered.contains("2. Bot 1 - 17 (out)"));
}

#[test]
fn summary_input_returns_to_menu_on_enter() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_state(GameState::RoundSummary)
        .init_resource::<GameData>()
        .insert_resource(ButtonInput::<KeyCode>::default())
        .add_systems(Update, summary_input.run_if(in_state(GameState::RoundSummary)));

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Enter);

    app.update();
    app.update();

    let state = app.world().resource::<State<GameState>>();
    assert!(matches!(state.get(), GameState::Menu));
}

#[test]
fn summary_input_can_start_new_round_with_difficulty_shortcuts() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_state(GameState::RoundSummary)
        .init_resource::<GameData>()
        .insert_resource(ButtonInput::<KeyCode>::default())
        .add_systems(Update, summary_input.run_if(in_state(GameState::RoundSummary)));

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Digit3);

    app.update();
    app.update();

    let state = app.world().resource::<State<GameState>>();
    assert!(matches!(state.get(), GameState::Playing));
    let data = app.world().resource::<GameData>();
    assert!(matches!(data.selected_difficulty, Difficulty::Hard));
}
