mod game;

use game::{Game, PostGameAction, TerminalGuard, render_post_game_menu, render_start_menu};

fn main() -> std::io::Result<()> {
    let _terminal_guard = TerminalGuard::activate()?;
    let mut selected = match render_start_menu()? {
        Some(settings) => settings,
        None => return Ok(()),
    };

    loop {
        let mut game = Game::new(selected);
        let summary = game.run()?;

        match render_post_game_menu(&summary, selected)? {
            PostGameAction::Replay => {}
            PostGameAction::Menu => {
                selected = match render_start_menu()? {
                    Some(settings) => settings,
                    None => break,
                };
            }
            PostGameAction::Quit => break,
        }
    }

    Ok(())
}
