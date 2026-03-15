use std::io::{Write, stdout};

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::queue;
use crossterm::style::{Color, ResetColor, SetForegroundColor};
use crossterm::terminal::{
    BeginSynchronizedUpdate, Clear, ClearType, EndSynchronizedUpdate, EnterAlternateScreen,
    LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};

use crate::game::core::Game;
use crate::game::types::{
    Cell, Difficulty, GameSettings, HEIGHT, PostGameAction, RoundSummary, WIDTH, in_bounds,
};

pub(crate) struct TerminalGuard;

impl TerminalGuard {
    pub(crate) fn activate() -> std::io::Result<Self> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen, Clear(ClearType::All), Hide)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(stdout(), Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

pub(crate) fn render_start_menu() -> std::io::Result<Option<GameSettings>> {
    let mut out = stdout();
    execute!(out, MoveTo(0, 0), Clear(ClearType::All))?;
    writeln!(out, "Snake IO (Rust)")?;
    writeln!(out, "Choose difficulty:")?;
    writeln!(out, "1. Easy   (2 bots, slower, more fruit)")?;
    writeln!(out, "2. Normal (3 bots, balanced)")?;
    writeln!(out, "3. Hard   (5 bots, faster, aggressive AI)")?;
    writeln!(out)?;
    writeln!(out, "Controls in match: Arrow keys move, Q quits")?;
    writeln!(out, "Press Q here to exit")?;
    out.flush()?;

    loop {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('1') => return Ok(Some(Difficulty::Easy.settings())),
                KeyCode::Char('2') => return Ok(Some(Difficulty::Normal.settings())),
                KeyCode::Char('3') => return Ok(Some(Difficulty::Hard.settings())),
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(None),
                _ => {}
            }
        }
    }
}

pub(crate) fn render_post_game_menu(
    _summary: &RoundSummary,
    _settings: GameSettings,
) -> std::io::Result<PostGameAction> {
    loop {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('r') | KeyCode::Char('R') => return Ok(PostGameAction::Replay),
                KeyCode::Char('m') | KeyCode::Char('M') => return Ok(PostGameAction::Menu),
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                    return Ok(PostGameAction::Quit);
                }
                _ => {}
            }
        }
    }
}

impl Game {
    pub(crate) fn render(&self) -> std::io::Result<()> {
        let mut out = stdout();
        queue!(out, BeginSynchronizedUpdate, MoveTo(0, 0))?;

        let player_score = self.player().map_or(0, |p| p.len());
        let alive_snakes = self.snakes.iter().filter(|s| s.alive).count();
        let mut line = 0u16;
        queue!(out, MoveTo(0, line), Clear(ClearType::CurrentLine))?;
        write!(
            out,
            "Snake IO (Rust) [{}]  Score: {player_score}  High Score: {}  Time Left: {}s  Alive Snakes: {alive_snakes}",
            self.settings.label(),
            self.high_score,
            self.remaining_seconds()
        )?;
        line += 1;

        queue!(out, MoveTo(0, line), Clear(ClearType::CurrentLine))?;
        write!(out, "Controls: Arrow keys move | Q or Esc quit")?;
        line += 1;

        let mut grid = vec![
            vec![
                Cell {
                    ch: ' ',
                    color: None,
                };
                WIDTH as usize
            ];
            HEIGHT as usize
        ];

        for fruit in &self.fruits {
            if in_bounds(*fruit) {
                grid[fruit.y as usize][fruit.x as usize] = Cell {
                    ch: '*',
                    color: Some(Color::Yellow),
                };
            }
        }

        for snake in self.snakes.iter().filter(|s| s.alive) {
            for (i, seg) in snake.body.iter().copied().enumerate() {
                if !in_bounds(seg) {
                    continue;
                }
                grid[seg.y as usize][seg.x as usize] = Cell {
                    ch: if i == 0 { snake.symbol } else { 'o' },
                    color: Some(snake.color),
                };
            }
        }

        queue!(out, MoveTo(0, line), Clear(ClearType::CurrentLine))?;
        write!(out, "+{}+", "-".repeat(WIDTH as usize))?;
        line += 1;

        for row in &grid {
            queue!(out, MoveTo(0, line), Clear(ClearType::CurrentLine))?;
            write!(out, "|")?;
            for c in row {
                if let Some(color) = c.color {
                    queue!(out, SetForegroundColor(color))?;
                    write!(out, "{}", c.ch)?;
                    queue!(out, ResetColor)?;
                } else {
                    write!(out, "{}", c.ch)?;
                }
            }
            write!(out, "|")?;
            line += 1;
        }

        queue!(out, MoveTo(0, line), Clear(ClearType::CurrentLine))?;
        write!(out, "+{}+", "-".repeat(WIDTH as usize))?;
        line += 1;

        let elapsed = self.start.elapsed().as_secs();
        for snake in &self.snakes {
            let status = if snake.alive {
                format!("alive | survival {}s", elapsed)
            } else {
                format!(
                    "out | survived {}s",
                    snake.eliminated_at.unwrap_or(elapsed)
                )
            };
            queue!(out, MoveTo(0, line), Clear(ClearType::CurrentLine))?;
            queue!(out, SetForegroundColor(snake.color))?;
            write!(
                out,
                "#{} {} [{}]: {} ({status})",
                snake.id,
                snake.name,
                snake.symbol,
                snake.len()
            )?;
            queue!(out, ResetColor)?;
            line += 1;
        }

        queue!(out, EndSynchronizedUpdate)?;
        out.flush()
    }

    pub(crate) fn render_game_over(&self, summary: &RoundSummary) -> std::io::Result<()> {
        let mut out = stdout();
        execute!(out, MoveTo(0, 0), Clear(ClearType::All))?;

        writeln!(out, "Game Over")?;
        writeln!(out, "Difficulty: {}", self.settings.label())?;
        writeln!(out, "Reason: {}", summary.end_reason)?;
        writeln!(out, "Winner: {}", summary.winner)?;
        writeln!(out, "High Score Achieved: {}", self.high_score)?;
        writeln!(out)?;
        writeln!(out, "Final Standings:")?;
        for (rank, index) in self.ranked_indices().iter().enumerate() {
            let snake = &self.snakes[*index];
            let status = if snake.alive {
                if self.remaining_seconds() == 0 {
                    "alive at clock end".to_string()
                } else {
                    "last alive".to_string()
                }
            } else {
                format!("eliminated at {}s", snake.eliminated_at.unwrap_or(0))
            };
            queue!(out, SetForegroundColor(snake.color))?;
            writeln!(
                out,
                "{}. #{} {}: length {} ({})",
                rank + 1,
                snake.id,
                snake.name,
                snake.len(),
                status
            )?;
            queue!(out, ResetColor)?;
        }
        writeln!(out)?;
        writeln!(out, "Press R to replay | M for menu | Q to quit")?;
        out.flush()
    }
}
