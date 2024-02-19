use std::{
    collections::{HashSet, VecDeque},
    fmt::Display,
    io::Write,
    ops::Add,
    sync::mpsc::{channel, Receiver},
    thread,
};

use console::{style, Key, Term};

use std::time::Instant;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    fn is_opposite(&self, other: Dir) -> bool {
        match (self, other) {
            (Dir::Up, Dir::Down) | (Dir::Down, Dir::Up) => true,
            (Dir::Left, Dir::Right) | (Dir::Right, Dir::Left) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TermPoint {
    pub row: usize,
    pub col: usize,
}

impl TermPoint {
    pub fn new(row: usize, col: usize) -> Self {
        TermPoint { row, col }
    }
}

impl Add<Dir> for TermPoint {
    type Output = Self;
    fn add(self, rhs: Dir) -> Self::Output {
        match rhs {
            Dir::Up => Self {
                row: self.row - 1,
                col: self.col,
            },
            Dir::Down => Self {
                row: self.row + 1,
                col: self.col,
            },
            Dir::Left => Self {
                row: self.row,
                col: self.col - 1,
            },
            Dir::Right => Self {
                row: self.row,
                col: self.col + 1,
            },
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BodySegment {
    pos: TermPoint,
    dir: Dir,
}

impl BodySegment {
    fn new(row: usize, col: usize, dir: Dir) -> Self {
        BodySegment {
            pos: TermPoint { row, col },
            dir,
        }
    }
}

impl Display for BodySegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let seg = match self.dir {
            Dir::Up => '^',
            Dir::Down => 'v',
            Dir::Left => '<',
            Dir::Right => '>',
        };
        write!(f, "{}", seg)?;
        Ok(())
    }
}

pub struct Snake {
    pub body: VecDeque<BodySegment>,
}

impl Snake {
    pub fn new() -> Self {
        Snake {
            body: VecDeque::new(),
        }
    }

    fn move_head(&mut self, dir: Dir) {
        let mut new_head: BodySegment = *self.body.front().unwrap();
        new_head.dir = dir;
        new_head.pos = new_head.pos + dir;

        self.body.push_front(new_head);
    }

    fn move_tail(&mut self) {
        self.body.pop_back();
    }

    pub fn move_body(&mut self, dir: Dir) {
        self.move_head(dir);
        self.move_tail();
    }

    pub fn extend_body(&mut self, new_tail: BodySegment) {
        self.body.push_back(new_tail);
    }
}

// TODO
#[allow(dead_code)]
pub struct GameSettings {
    // todo...
}

pub struct SnakeGame {
    term: Term,
    input_rcv: Receiver<Key>,
    snake: Snake,
    score: usize,
    open_space: HashSet<TermPoint>,
    apple: TermPoint,
}

#[allow(dead_code)]
impl GameSettings {
    pub fn new() -> Self {
        GameSettings {}
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UserInput {
    Unknown,
    Pause,
    Up,
    Down,
    Left,
    Right,
}

impl From<Key> for UserInput {
    fn from(value: Key) -> Self {
        match value {
            Key::ArrowLeft => Self::Left,
            Key::ArrowRight => Self::Right,
            Key::ArrowUp => Self::Up,
            Key::ArrowDown => Self::Down,
            Key::Escape => Self::Pause,
            _ => Self::Unknown,
        }
    }
}

impl From<UserInput> for Dir {
    fn from(value: UserInput) -> Self {
        match value {
            UserInput::Up => Self::Up,
            UserInput::Down => Self::Down,
            UserInput::Left => Self::Left,
            UserInput::Right => Self::Right,
            _ => Self::Down,
        }
    }
}

impl From<Dir> for UserInput {
    fn from(value: Dir) -> Self {
        match value {
            Dir::Up => Self::Up,
            Dir::Down => Self::Down,
            Dir::Left => Self::Left,
            Dir::Right => Self::Right,
        }
    }
}

impl SnakeGame {
    pub fn new(term: Term, input_rcv: Receiver<Key>) -> Self {
        let mut snake = Snake::new();
        snake.body.push_back(BodySegment::new(1, 1, Dir::Right));
        snake.body.push_back(BodySegment::new(1, 2, Dir::Right));
        let score = 0usize;
        let apple = TermPoint::new(1, 5);

        let mut open_space: HashSet<TermPoint> = HashSet::new();

        let (ht, wt) = term.size();
        let height = ht as usize;
        let width = wt as usize;
        for col in 1..width - 1 {
            for row in 1..height - 1 {
                open_space.insert(TermPoint::new(row, col));
            }
        }

        for seg in snake.body.iter() {
            open_space.remove(&seg.pos);
        }

        SnakeGame {
            term,
            input_rcv,
            snake,
            score,
            open_space,
            apple,
        }
    }

    fn add_apple(&mut self) {
        let idx = rand::random::<usize>() % self.open_space.len();
        self.apple = *self.open_space.iter().nth(idx).unwrap();
    }

    // add pausing here?
    pub fn update_state(&mut self, input: UserInput) -> anyhow::Result<GameState> {
        let (ht, wt) = self.term.size();
        let height = ht as usize;
        let width = wt as usize;

        let old_tail = *self.snake.body.back().unwrap();
        self.snake.move_body(input.into());
        self.open_space
            .remove(&self.snake.body.front().unwrap().pos);
        // edge collision check
        let head = self.snake.body.front().unwrap().pos;
        if head.row == 0 || head.row >= height - 1 || head.col == 0 || head.col >= width - 1 {
            return Ok(GameState::Over);
        }
        // self collision check
        for seg in self.snake.body.iter().skip(1) {
            if seg.pos == head {
                return Ok(GameState::Over);
            }
        }

        if self.snake.body.front().unwrap().pos == self.apple {
            if self.open_space.is_empty() {
                return Ok(GameState::Win);
            }
            self.snake.extend_body(old_tail);
            self.score += 100;
            self.add_apple();
        } else {
            self.open_space.insert(old_tail.pos);
        }
        Ok(GameState::Continue)
    }

    fn render(&mut self) -> anyhow::Result<()> {
        self.term.clear_screen()?;
        // draw border
        let (ht, wt) = self.term.size();
        let height = ht as usize;
        let width = wt as usize;

        let border_block = "█";
        let top_border = border_block.repeat(width);
        self.term.move_cursor_to(0, 0)?;
        self.term.write_all(top_border.as_bytes())?;
        self.term.move_cursor_to(0, height - 1)?;
        self.term.write_all(top_border.as_bytes())?;
        // score
        self.term.move_cursor_to(0, height - 1)?;
        let score_str = format!(
            "{}{}",
            style("Score: ").black().on_white(),
            style(self.score).black().on_white()
        );
        self.term.write_all(score_str.as_bytes())?;
        for row in 1..height - 1 {
            self.term.move_cursor_to(0, row)?;
            self.term.write_all(border_block.as_bytes())?;
            self.term.move_cursor_to(width - 1, row)?;
            self.term.write_all(border_block.as_bytes())?;
        }

        // draw apple
        self.term.move_cursor_to(self.apple.col, self.apple.row)?;
        let apple = format!("{}", style("O").red().on_black());
        self.term.write_all(apple.as_bytes())?;

        // draw snake
        for part in self.snake.body.iter() {
            self.term.move_cursor_to(part.pos.col, part.pos.row)?;
            let seg = format!("{}", style(part).green().on_white());
            self.term.write_all(seg.as_bytes())?;
        }

        Ok(())
    }
}

pub enum GameState {
    Continue,
    Over,
    Win,
}

pub fn play(term: Term) -> anyhow::Result<()> {
    let tx_term = term.clone();
    let (tx, rx) = channel();
    thread::spawn(move || loop {
        let key = tx_term.read_key().unwrap();
        tx.send(key).unwrap();
    });
    let mut game_state = SnakeGame::new(term.clone(), rx);
    let mut user_in = UserInput::Right;

    loop {
        game_state.render()?;
        let start = Instant::now();
        while start.elapsed().as_secs_f64() < 0.0625 {
            match game_state.input_rcv.try_recv() {
                Ok(key) => {
                    user_in = key.into();
                }
                Err(_e) => {}
            }
        }
        if game_state
            .snake
            .body
            .front()
            .unwrap()
            .dir
            .is_opposite(user_in.into())
        {
            user_in = game_state.snake.body.front().unwrap().dir.into();
        }
        match game_state.update_state(user_in) {
            Ok(GameState::Over) => {
                let msg = format!("Game Over: {}", game_state.score);
                game_state.term.write_all(msg.as_bytes())?;
                break;
            }
            Ok(GameState::Continue) => {}
            _ => {
                game_state.term.write_all("Uh oh".as_bytes())?;
                break;
            }
        }
    }

    Ok(())
}
