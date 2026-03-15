use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{Write, stdout};
use std::thread;
use std::time::{Duration, Instant};

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::queue;
use crossterm::style::{Color, ResetColor, SetForegroundColor};
use crossterm::terminal::{
    BeginSynchronizedUpdate, Clear, ClearType, EndSynchronizedUpdate, EnterAlternateScreen,
    LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use rand::Rng;

const WIDTH: i32 = 48;
const HEIGHT: i32 = 20;
const MAX_BOTS: usize = 6;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn add(self, d: Direction) -> Self {
        let (dx, dy) = d.delta();
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn delta(self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }

    fn opposite(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

struct Snake {
    id: usize,
    name: String,
    body: VecDeque<Point>,
    dir: Direction,
    next_dir: Direction,
    pending_growth: usize,
    is_player: bool,
    alive: bool,
    symbol: char,
    color: Color,
}

#[derive(Clone, Copy)]
struct Cell {
    ch: char,
    color: Option<Color>,
}

#[derive(Clone, Copy)]
enum Difficulty {
    Easy,
    Normal,
    Hard,
}

#[derive(Clone, Copy)]
struct GameSettings {
    difficulty: Difficulty,
    fruit_count: usize,
    bot_count: usize,
    tick_ms: u64,
    game_time_seconds: u64,
    bot_aggression: f32,
}

impl GameSettings {
    fn label(self) -> &'static str {
        match self.difficulty {
            Difficulty::Easy => "Easy",
            Difficulty::Normal => "Normal",
            Difficulty::Hard => "Hard",
        }
    }
}

impl Difficulty {
    fn settings(self) -> GameSettings {
        match self {
            Difficulty::Easy => GameSettings {
                difficulty: self,
                fruit_count: 10,
                bot_count: 2,
                tick_ms: 145,
                game_time_seconds: 75,
                bot_aggression: 0.55,
            },
            Difficulty::Normal => GameSettings {
                difficulty: self,
                fruit_count: 8,
                bot_count: 3,
                tick_ms: 120,
                game_time_seconds: 90,
                bot_aggression: 0.85,
            },
            Difficulty::Hard => GameSettings {
                difficulty: self,
                fruit_count: 6,
                bot_count: 5,
                tick_ms: 95,
                game_time_seconds: 100,
                bot_aggression: 1.25,
            },
        }
    }
}

enum PostGameAction {
    Replay,
    Menu,
    Quit,
}

struct RoundSummary {
    winner: String,
    end_reason: String,
}

impl Snake {
    fn len(&self) -> usize {
        self.body.len()
    }

    fn head(&self) -> Option<Point> {
        self.body.front().copied()
    }

    fn set_direction(&mut self, new_dir: Direction) {
        if new_dir != self.dir.opposite() {
            self.next_dir = new_dir;
        }
    }

    fn move_forward(&mut self) {
        if !self.alive {
            return;
        }
        self.dir = self.next_dir;
        if let Some(head) = self.head() {
            self.body.push_front(head.add(self.dir));
            if self.pending_growth > 0 {
                self.pending_growth -= 1;
            } else {
                self.body.pop_back();
            }
        }
    }
}

struct Game {
    settings: GameSettings,
    snakes: Vec<Snake>,
    fruits: Vec<Point>,
    rng: rand::rngs::ThreadRng,
    start: Instant,
    high_score: usize,
    quit_requested: bool,
}

struct TerminalGuard;

impl TerminalGuard {
    fn activate() -> std::io::Result<Self> {
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

impl Game {
    fn new(settings: GameSettings) -> Self {
        let mut game = Self {
            settings,
            snakes: Vec::new(),
            fruits: Vec::new(),
            rng: rand::rng(),
            start: Instant::now(),
            high_score: 0,
            quit_requested: false,
        };

        game.spawn_snakes();
        game.refill_fruits();
        game
    }

    fn run(&mut self) -> std::io::Result<RoundSummary> {
        while !self.is_finished() {
            let frame_start = Instant::now();
            self.process_input()?;
            self.update_bot_directions();
            self.move_snakes();
            self.resolve_head_to_head();
            self.resolve_wall_and_self_collisions();
            self.resolve_snake_eating();
            self.resolve_cross_snake_overlaps();
            self.resolve_fruit_eating();
            self.update_high_score();
            self.refill_fruits();
            self.render()?;

            let elapsed = frame_start.elapsed();
            let tick = self.current_tick_duration();
            if elapsed < tick {
                thread::sleep(tick - elapsed);
            }
        }

        let summary = self.build_summary();
        self.render_game_over(&summary)?;
        Ok(summary)
    }

    fn is_finished(&self) -> bool {
        self.quit_requested
            || self.elapsed_seconds() >= self.settings.game_time_seconds
            || !self.player().is_some_and(|p| p.alive)
    }

    fn elapsed_seconds(&self) -> u64 {
        self.start.elapsed().as_secs()
    }

    fn remaining_seconds(&self) -> u64 {
        self.settings
            .game_time_seconds
            .saturating_sub(self.elapsed_seconds())
    }

    fn current_tick_duration(&self) -> Duration {
        let base = self.settings.tick_ms;
        let min_tick = ((base as f32) * 0.55) as u64;
        let player_len = self.player().map_or(3usize, |p| p.len());
        let growth = player_len.saturating_sub(3) as u64;
        // Increase speed slowly: reduce 1ms every 2 growth points.
        let reduction = growth / 2;
        let tick_ms = base.saturating_sub(reduction).max(min_tick);
        Duration::from_millis(tick_ms)
    }

    fn player(&self) -> Option<&Snake> {
        self.snakes.iter().find(|s| s.is_player)
    }

    fn player_mut(&mut self) -> Option<&mut Snake> {
        self.snakes.iter_mut().find(|s| s.is_player)
    }

    fn spawn_snakes(&mut self) {
        self.snakes.push(make_snake(
            0,
            "Player",
            Point {
                x: WIDTH / 4,
                y: HEIGHT / 2,
            },
            Direction::Right,
            true,
            '@',
            Color::Cyan,
        ));

        let bot_count = self.settings.bot_count.min(MAX_BOTS);
        for i in 0..bot_count {
            let lane = i as i32 + 1;
            let y = lane * HEIGHT / (bot_count as i32 + 1);
            let x = if i % 2 == 0 { WIDTH * 3 / 4 } else { WIDTH * 3 / 4 + 1 };
            let symbol = char::from_u32('A' as u32 + i as u32).unwrap_or('B');
            self.snakes.push(make_snake(
                i + 1,
                &format!("Bot {}", i + 1),
                Point { x, y },
                Direction::Left,
                false,
                symbol,
                bot_color(i),
            ));
        }

        self.update_high_score();
    }

    fn process_input(&mut self) -> std::io::Result<()> {
        while event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up => {
                        if let Some(p) = self.player_mut() {
                            p.set_direction(Direction::Up);
                        }
                    }
                    KeyCode::Down => {
                        if let Some(p) = self.player_mut() {
                            p.set_direction(Direction::Down);
                        }
                    }
                    KeyCode::Left => {
                        if let Some(p) = self.player_mut() {
                            p.set_direction(Direction::Left);
                        }
                    }
                    KeyCode::Right => {
                        if let Some(p) = self.player_mut() {
                            p.set_direction(Direction::Right);
                        }
                    }
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                        self.quit_requested = true;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn update_bot_directions(&mut self) {
        for i in 0..self.snakes.len() {
            if self.snakes[i].is_player || !self.snakes[i].alive {
                continue;
            }

            let current_head = match self.snakes[i].head() {
                Some(h) => h,
                None => continue,
            };

            let mut candidates = [
                self.snakes[i].dir,
                Direction::Up,
                Direction::Right,
                Direction::Down,
                Direction::Left,
            ];

            candidates.sort_by(|a, b| {
                let sa = self.direction_score(i, current_head, *a);
                let sb = self.direction_score(i, current_head, *b);
                sb.partial_cmp(&sa).unwrap_or(Ordering::Equal)
            });

            for dir in candidates {
                if dir == self.snakes[i].dir.opposite() {
                    continue;
                }
                let next = current_head.add(dir);
                if !self.bot_can_step_into(i, next) {
                    continue;
                }
                self.snakes[i].set_direction(dir);
                break;
            }
        }
    }

    fn direction_score(&self, snake_index: usize, head: Point, dir: Direction) -> f32 {
        if dir == self.snakes[snake_index].dir.opposite() {
            return -10_000.0;
        }
        let next = head.add(dir);
        if next.x < 0 || next.x >= WIDTH || next.y < 0 || next.y >= HEIGHT {
            return -5_000.0;
        }

        let mut score = 0.0;
        if !self.bot_can_step_into(snake_index, next) {
            score -= 1_400.0;
        }

        if dir == self.snakes[snake_index].dir {
            score += 2.0;
        }

        if let Some(target) = self.closest_fruit(next) {
            let dist = manhattan(next, target) as f32;
            score += 62.0 / (1.0 + dist);
        }

        score += self.bot_hunt_score(snake_index, next);
        score += self.attack_cell_bonus(snake_index, next);
        score += self.local_open_space_score(snake_index, next);

        let center = Point {
            x: WIDTH / 2,
            y: HEIGHT / 2,
        };
        score += 10.0 / (1.0 + manhattan(next, center) as f32);
        score + rand::random_range(0.0f32..1.0f32)
    }

    fn bot_hunt_score(&self, snake_index: usize, next: Point) -> f32 {
        let my_len = self.snakes[snake_index].len() as f32;
        let mut score = 0.0;

        for (idx, snake) in self.snakes.iter().enumerate() {
            if idx == snake_index || !snake.alive {
                continue;
            }
            let Some(head) = snake.head() else {
                continue;
            };
            let distance = manhattan(next, head) as f32;

            if snake.len() + 1 < self.snakes[snake_index].len() {
                score += (22.0 * self.settings.bot_aggression) / (1.0 + distance);
            } else if snake.len() > my_len as usize {
                score -= 28.0 / (1.0 + distance);
            } else {
                score -= 6.0 / (1.0 + distance);
            }
        }

        score
    }

    fn attack_cell_bonus(&self, snake_index: usize, next: Point) -> f32 {
        let mut bonus = 0.0;
        let my_len = self.snakes[snake_index].len();

        for (idx, snake) in self.snakes.iter().enumerate() {
            if idx == snake_index || !snake.alive {
                continue;
            }

            for (seg_index, segment) in snake.body.iter().copied().enumerate() {
                if segment != next {
                    continue;
                }

                if seg_index == 0 {
                    if my_len > snake.len() {
                        bonus += 180.0 * self.settings.bot_aggression;
                    } else {
                        bonus -= 240.0;
                    }
                } else {
                    let bite_amount = (snake.len() - seg_index) as f32;
                    bonus += (30.0 + bite_amount * 8.0) * self.settings.bot_aggression;
                }
            }
        }

        bonus
    }

    fn local_open_space_score(&self, snake_index: usize, origin: Point) -> f32 {
        let neighbors = [Direction::Up, Direction::Right, Direction::Down, Direction::Left];
        let mut safe_count = 0.0;
        for dir in neighbors {
            let p = origin.add(dir);
            if self.bot_can_step_into(snake_index, p) {
                safe_count += 1.0;
            }
        }
        safe_count * 4.0
    }

    fn bot_can_step_into(&self, snake_index: usize, cell: Point) -> bool {
        if cell.x < 0 || cell.x >= WIDTH || cell.y < 0 || cell.y >= HEIGHT {
            return false;
        }

        let my_len = self.snakes[snake_index].len();

        for (idx, snake) in self.snakes.iter().enumerate() {
            if !snake.alive {
                continue;
            }

            for (seg_index, segment) in snake.body.iter().enumerate() {
                if *segment != cell {
                    continue;
                }

                if idx == snake_index {
                    if seg_index == snake.len().saturating_sub(1) && snake.pending_growth == 0 {
                        continue;
                    }
                    return false;
                }

                if seg_index == 0 {
                    return my_len > snake.len();
                }

                return true;
            }
        }

        true
    }

    fn closest_fruit(&self, from: Point) -> Option<Point> {
        self.fruits
            .iter()
            .copied()
            .min_by_key(|fruit| manhattan(from, *fruit))
    }

    fn move_snakes(&mut self) {
        for snake in &mut self.snakes {
            snake.move_forward();
        }
    }

    fn resolve_head_to_head(&mut self) {
        let mut positions: HashMap<Point, Vec<usize>> = HashMap::new();
        for (i, snake) in self.snakes.iter().enumerate() {
            if !snake.alive {
                continue;
            }
            if let Some(h) = snake.head() {
                positions.entry(h).or_default().push(i);
            }
        }

        for indices in positions.into_values() {
            if indices.len() <= 1 {
                continue;
            }

            let mut strongest_len = 0usize;
            for i in &indices {
                strongest_len = strongest_len.max(self.snakes[*i].len());
            }

            let strongest_indices: Vec<usize> = indices
                .iter()
                .copied()
                .filter(|i| self.snakes[*i].len() == strongest_len)
                .collect();

            let player_wins_tie = strongest_indices
                .iter()
                .any(|i| self.snakes[*i].is_player);

            if strongest_indices.len() > 1 && !player_wins_tie {
                for i in indices {
                    self.snakes[i].alive = false;
                }
            } else {
                let winner = if player_wins_tie {
                    strongest_indices
                        .iter()
                        .copied()
                        .find(|i| self.snakes[*i].is_player)
                        .unwrap_or(strongest_indices[0])
                } else {
                    strongest_indices[0]
                };
                let mut growth = 0usize;
                for i in indices {
                    if i != winner {
                        growth += self.snakes[i].len();
                        self.snakes[i].alive = false;
                    }
                }
                self.snakes[winner].pending_growth += growth;
            }
        }
    }

    fn resolve_wall_and_self_collisions(&mut self) {
        for snake in &mut self.snakes {
            if !snake.alive {
                continue;
            }
            let head = match snake.head() {
                Some(h) => h,
                None => continue,
            };

            if head.x < 0 || head.x >= WIDTH || head.y < 0 || head.y >= HEIGHT {
                snake.alive = false;
                continue;
            }

            for segment in snake.body.iter().skip(1) {
                if *segment == head {
                    snake.alive = false;
                    break;
                }
            }
        }
    }

    fn resolve_snake_eating(&mut self) {
        let mut interactions = Vec::new();

        for eater_index in 0..self.snakes.len() {
            if !self.snakes[eater_index].alive {
                continue;
            }
            let eater_head = match self.snakes[eater_index].head() {
                Some(h) => h,
                None => continue,
            };

            for victim_index in 0..self.snakes.len() {
                if eater_index == victim_index || !self.snakes[victim_index].alive {
                    continue;
                }

                for (seg_index, segment) in self.snakes[victim_index].body.iter().copied().enumerate() {
                    if segment == eater_head {
                        interactions.push((eater_index, victim_index, seg_index));
                        break;
                    }
                }
            }
        }

        for (eater, victim, seg_index) in interactions {
            if !self.snakes[eater].alive || !self.snakes[victim].alive {
                continue;
            }

            if seg_index == 0 {
                let victim_len = self.snakes[victim].len();
                self.snakes[victim].alive = false;
                self.snakes[eater].pending_growth += victim_len;
            } else {
                let victim_len = self.snakes[victim].len();
                if seg_index >= victim_len {
                    continue;
                }
                // Keep a one-cell gap from the bite point so the snakes do not appear merged.
                let kept_len = seg_index.saturating_sub(1);
                let eaten_amount = victim_len - kept_len;
                self.snakes[victim].body.truncate(kept_len);
                if self.snakes[victim].body.is_empty() {
                    self.snakes[victim].alive = false;
                }
                self.snakes[eater].pending_growth += eaten_amount;
            }
        }
    }

    fn resolve_cross_snake_overlaps(&mut self) {
        // Resolve any shared cell between different snakes so bodies never visually merge.
        for _ in 0..3 {
            let mut occupancy: HashMap<Point, Vec<(usize, usize)>> = HashMap::new();
            for (snake_index, snake) in self.snakes.iter().enumerate() {
                if !snake.alive {
                    continue;
                }
                for (seg_index, segment) in snake.body.iter().copied().enumerate() {
                    occupancy
                        .entry(segment)
                        .or_default()
                        .push((snake_index, seg_index));
                }
            }

            let mut had_overlap = false;
            for entries in occupancy.into_values() {
                let alive_distinct = entries
                    .iter()
                    .filter(|(idx, _)| self.snakes[*idx].alive)
                    .map(|(idx, _)| *idx)
                    .collect::<HashSet<_>>();

                if alive_distinct.len() <= 1 {
                    continue;
                }
                had_overlap = true;

                let has_head = entries
                    .iter()
                    .any(|(idx, seg)| self.snakes[*idx].alive && *seg == 0);

                let owner = entries
                    .iter()
                    .filter(|(idx, _)| self.snakes[*idx].alive)
                    .max_by_key(|(idx, seg)| {
                        let head_priority = if has_head && *seg == 0 { 1usize } else { 0usize };
                        (head_priority, self.snakes[*idx].len(), usize::MAX - self.snakes[*idx].id)
                    })
                    .map(|(idx, _)| *idx);

                let Some(owner_index) = owner else {
                    continue;
                };

                for (snake_index, seg_index) in entries {
                    if snake_index == owner_index || !self.snakes[snake_index].alive {
                        continue;
                    }

                    if seg_index == 0 {
                        let victim_len = self.snakes[snake_index].len();
                        self.snakes[snake_index].alive = false;
                        self.snakes[owner_index].pending_growth += victim_len;
                    } else {
                        let victim_len = self.snakes[snake_index].len();
                        let kept_len = seg_index.saturating_sub(1);
                        let eaten_amount = victim_len.saturating_sub(kept_len);
                        self.snakes[snake_index].body.truncate(kept_len);
                        if self.snakes[snake_index].body.is_empty() {
                            self.snakes[snake_index].alive = false;
                        }
                        self.snakes[owner_index].pending_growth += eaten_amount;
                    }
                }
            }

            if !had_overlap {
                break;
            }
        }
    }

    fn resolve_fruit_eating(&mut self) {
        let mut consumed = Vec::new();
        for (fi, fruit) in self.fruits.iter().copied().enumerate() {
            for snake in &mut self.snakes {
                if !snake.alive {
                    continue;
                }
                if snake.head().is_some_and(|h| h == fruit) {
                    snake.pending_growth += 1;
                    consumed.push(fi);
                    break;
                }
            }
        }

        consumed.sort_unstable();
        consumed.dedup();
        while let Some(index) = consumed.pop() {
            self.fruits.remove(index);
        }
    }

    fn refill_fruits(&mut self) {
        while self.fruits.len() < self.settings.fruit_count {
            if let Some(p) = self.random_empty_cell() {
                self.fruits.push(p);
            } else {
                break;
            }
        }
    }

    fn random_empty_cell(&mut self) -> Option<Point> {
        for _ in 0..500 {
            let p = Point {
                x: self.rng.random_range(0..WIDTH),
                y: self.rng.random_range(0..HEIGHT),
            };

            if self.fruits.contains(&p) {
                continue;
            }

            if self
                .snakes
                .iter()
                .filter(|s| s.alive)
                .any(|s| s.body.iter().any(|seg| *seg == p))
            {
                continue;
            }

            return Some(p);
        }
        None
    }

    fn update_high_score(&mut self) {
        for snake in &self.snakes {
            self.high_score = self.high_score.max(snake.len());
        }
    }

    fn render(&self) -> std::io::Result<()> {
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

        for snake in &self.snakes {
            let status = if snake.alive { "alive" } else { "out" };
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

    fn build_summary(&self) -> RoundSummary {
        let winner = self
            .snakes
            .iter()
            .max_by_key(|s| s.len())
            .map(|s| format!("{} (length {})", s.name, s.len()))
            .unwrap_or_else(|| "No winner".to_string());

        let player_alive = self.player().is_some_and(|s| s.alive);
        let end_reason = if self.quit_requested {
            "Quit requested".to_string()
        } else if !player_alive {
            "Player collided and was eliminated".to_string()
        } else {
            "Timer ended".to_string()
        };

        RoundSummary { winner, end_reason }
    }

    fn render_game_over(&self, summary: &RoundSummary) -> std::io::Result<()> {
        let mut out = stdout();
        execute!(out, MoveTo(0, 0), Clear(ClearType::All))?;

        writeln!(out, "Game Over")?;
        writeln!(out, "Difficulty: {}", self.settings.label())?;
        writeln!(out, "Reason: {}", summary.end_reason)?;
        writeln!(out, "Winner: {}", summary.winner)?;
        writeln!(out, "High Score Achieved: {}", self.high_score)?;
        writeln!(out)?;
        writeln!(out, "Final Standings:")?;
        for snake in &self.snakes {
            queue!(out, SetForegroundColor(snake.color))?;
            writeln!(out, "- #{} {}: {}", snake.id, snake.name, snake.len())?;
            queue!(out, ResetColor)?;
        }
        writeln!(out)?;
        writeln!(out, "Press R to replay | M for menu | Q to quit")?;
        out.flush()
    }
}

fn render_start_menu() -> std::io::Result<Option<GameSettings>> {
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

fn render_post_game_menu(
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

fn in_bounds(p: Point) -> bool {
    p.x >= 0 && p.x < WIDTH && p.y >= 0 && p.y < HEIGHT
}

fn manhattan(a: Point, b: Point) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

fn make_snake(
    id: usize,
    name: &str,
    start: Point,
    dir: Direction,
    is_player: bool,
    symbol: char,
    color: Color,
) -> Snake {
    let mut body = VecDeque::new();
    body.push_back(start);
    body.push_back(start.add(dir.opposite()));
    body.push_back(start.add(dir.opposite()).add(dir.opposite()));

    Snake {
        id,
        name: name.to_string(),
        body,
        dir,
        next_dir: dir,
        pending_growth: 0,
        is_player,
        alive: true,
        symbol,
        color,
    }
}

fn bot_color(index: usize) -> Color {
    match index % 6 {
        0 => Color::Red,
        1 => Color::Green,
        2 => Color::Magenta,
        3 => Color::Blue,
        4 => Color::DarkYellow,
        _ => Color::DarkCyan,
    }
}
