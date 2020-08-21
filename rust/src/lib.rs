use std::convert::{Into, TryInto};
use std::io::{self, Read};

use ansi_term::Colour::{Green, Red, White, Yellow};
use anyhow::{anyhow, bail, Context, Result};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Not,
    Gt,
    Right,
    Left,
    Up,
    Down,
    Rand,
    IfH,
    IfV,
    Str,
    Dup,
    Swap,
    Pop,
    OutI,
    OutC,
    Bri,
    Get,
    Put,
    InI,
    InC,
    End,
    Space,
    Num(u8),
    Char(char),
}

impl Command {
    fn as_char(&self) -> char {
        match self {
            Self::Add => '+',
            Self::Sub => '-',
            Self::Mul => '*',
            Self::Div => '/',
            Self::Mod => '%',
            Self::Not => '!',
            Self::Gt => '`',
            Self::Right => '>',
            Self::Left => '<',
            Self::Up => '^',
            Self::Down => 'v',
            Self::Rand => '?',
            Self::IfH => '_',
            Self::IfV => '|',
            Self::Str => '"',
            Self::Dup => ':',
            Self::Swap => '\\',
            Self::Pop => '$',
            Self::OutI => '.',
            Self::OutC => ',',
            Self::Bri => '#',
            Self::Get => 'g',
            Self::Put => 'p',
            Self::InI => '&',
            Self::InC => '~',
            Self::End => '@',
            Self::Space => ' ',
            Self::Num(n) => (n + 48) as char,
            Self::Char(c) => *c,
        }
    }
}

impl From<char> for Command {
    fn from(c: char) -> Self {
        match c {
            '+' => Self::Add,
            '-' => Self::Sub,
            '*' => Self::Mul,
            '/' => Self::Div,
            '%' => Self::Mod,
            '!' => Self::Not,
            '`' => Self::Gt,
            '>' => Self::Right,
            '<' => Self::Left,
            '^' => Self::Up,
            'v' => Self::Down,
            '?' => Self::Rand,
            '_' => Self::IfH,
            '|' => Self::IfV,
            '"' => Self::Str,
            ':' => Self::Dup,
            '\\' => Self::Swap,
            '$' => Self::Pop,
            '.' => Self::OutI,
            ',' => Self::OutC,
            '#' => Self::Bri,
            'g' => Self::Get,
            'p' => Self::Put,
            '&' => Self::InI,
            '~' => Self::InC,
            '@' => Self::End,
            ' ' => Self::Space,
            '0'..='9' => Self::Num(c.to_digit(10).unwrap() as u8),
            _ => Self::Char(c),
        }
    }
}

impl Into<char> for Command {
    fn into(self) -> char {
        self.as_char()
    }
}

impl ToString for Command {
    fn to_string(&self) -> String {
        self.as_char().to_string()
    }
}

#[derive(Debug)]
struct ProgramCounter {
    x: usize,
    y: usize,
}

impl ProgramCounter {
    fn init() -> Self {
        Self { x: 0, y: 0 }
    }

    fn reset(&mut self) {
        self.x = 0;
        self.y = 0;
    }

    fn right(&mut self) {
        self.x = (self.x + 1) % PLAYFIELD_COLS;
    }

    fn left(&mut self) {
        if self.x == 0 {
            self.x = PLAYFIELD_COLS - 1;
        } else {
            self.x -= 1;
        }
    }

    fn down(&mut self) {
        self.y = (self.y + 1) % PLAYFIELD_ROWS;
    }

    fn up(&mut self) {
        if self.y == 0 {
            self.y = PLAYFIELD_ROWS - 1;
        } else {
            self.y -= 1;
        }
    }
}

const PLAYFIELD_ROWS: usize = 25;
const PLAYFIELD_COLS: usize = 80;

type StackTy = i64;

#[derive(Clone)]
pub struct Stack(Vec<StackTy>);

impl Stack {
    fn reset(&mut self) {
        self.0.clear();
    }

    fn pop(&mut self) -> StackTy {
        self.0.pop().unwrap_or(0)
    }

    fn push(&mut self, val: StackTy) {
        self.0.push(val);
    }

    fn peek(&self) -> StackTy {
        *self.0.last().unwrap_or(&0)
    }
}

impl ToString for Stack {
    fn to_string(&self) -> String {
        let mut s = String::new();
        for item in &self.0 {
            let s0 = Green.on(White).paint(item.to_string() + " ");
            s += &s0;
        }
        s
    }
}

#[derive(Copy, Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(PartialEq)]
enum StepResult {
    Cont,
    Stop,
}

pub struct Interpreter {
    /// The playfield to work on. Acts as code and data storage.
    playfield: [[Command; PLAYFIELD_COLS]; PLAYFIELD_ROWS],
    /// The program counter.
    pc: ProgramCounter,
    /// The direction the PC is moving.
    dir: Direction,
    /// The stack.
    stack: Stack,
    /// Whether string mode is active.
    stringmode: bool,
    /// The PRNG used for `?`.
    rng: SmallRng,
    /// The current output.
    output: String,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    /// Create a new empty interpreter.
    pub fn new() -> Self {
        Self {
            playfield: [[Command::Space; PLAYFIELD_COLS]; PLAYFIELD_ROWS],
            pc: ProgramCounter::init(),
            dir: Direction::Right,
            stack: Stack(vec![]),
            stringmode: false,
            rng: SmallRng::from_entropy(),
            output: String::new(),
        }
    }

    /// Load playfield from reader.
    pub fn load(&mut self, reader: &mut impl io::Read) -> Result<()> {
        let mut buf = vec![];
        reader.read_to_end(&mut buf)?;

        let (mut x, mut y) = (0, 0);
        for item in buf {
            if item == b'\n' {
                x = 0;
                y = (y + 1) % PLAYFIELD_ROWS;
                continue;
            }

            self.playfield[y][x] = Command::from(item as char);

            x = (x + 1) % PLAYFIELD_COLS;
            if x == 0 {
                y = (y + 1) % PLAYFIELD_ROWS;
            }
        }

        Ok(())
    }

    /// Get a copy of the current stack.
    pub fn get_stack(&self) -> Stack {
        self.stack.clone()
    }

    /// Inspect the current output.
    pub fn get_output(&self) -> &str {
        &self.output
    }

    /// Get the current command.
    pub fn get_current_command(&self) -> Command {
        self.playfield[self.pc.y][self.pc.x]
    }

    fn binop<F: Fn(StackTy, StackTy) -> StackTy>(&mut self, f: F) {
        let y = self.stack.pop();
        let x = self.stack.pop();
        self.stack.push(f(x, y));
    }

    fn step(&mut self) -> Result<StepResult> {
        let cmd = self.playfield[self.pc.y][self.pc.x];

        if self.stringmode {
            if let Command::Str = cmd {
                self.stringmode = false;
            } else {
                self.stack.push((cmd.as_char() as u8).into());
            }

            self.advance_pc();
            return Ok(StepResult::Cont);
        }

        match cmd {
            Command::Add => self.binop(|x, y| x + y),
            Command::Sub => self.binop(|x, y| x - y),
            Command::Mul => self.binop(|x, y| x * y),
            Command::Div => self.binop(|x, y| x / y),
            Command::Mod => self.binop(|x, y| x % y),
            Command::Not => {
                let x = self.stack.pop();
                self.stack.push(if x == 0 { 1 } else { 0 });
            }
            Command::Gt => self.binop(|x, y| if x > y { 1 } else { 0 }),
            Command::Right => self.dir = Direction::Right,
            Command::Left => self.dir = Direction::Left,
            Command::Up => self.dir = Direction::Up,
            Command::Down => self.dir = Direction::Down,
            Command::Rand => {
                self.dir = [
                    Direction::Up,
                    Direction::Down,
                    Direction::Left,
                    Direction::Right,
                ][self.rng.gen_range(0, 4)];
            }
            Command::IfH => {
                let x = self.stack.pop();
                self.dir = if x == 0 {
                    Direction::Right
                } else {
                    Direction::Left
                };
            }
            Command::IfV => {
                let x = self.stack.pop();
                self.dir = if x == 0 {
                    Direction::Down
                } else {
                    Direction::Up
                };
            }
            Command::Str => self.stringmode = !self.stringmode,
            Command::Dup => self.stack.push(self.stack.peek()),
            Command::Swap => {
                let x = self.stack.pop();
                let y = self.stack.pop();
                self.stack.push(x);
                self.stack.push(y);
            }
            Command::Pop => {
                self.stack.pop();
            }
            Command::OutI => {
                let x = self.stack.pop();
                self.output += &format!("{} ", x);
            }
            Command::OutC => {
                let x = self.stack.pop();
                self.output += &format!("{}", x as u8 as char);
            }
            Command::InI => {
                let mut stdin = io::stdin();
                let mut buf = [0; 1];
                let mut s = String::new();
                loop {
                    stdin.read_exact(&mut buf).context("Reading a byte")?;
                    if buf[0] == b' ' {
                        break;
                    }
                    s.push(buf[0] as char);
                }
                self.stack.push(
                    s.parse()
                        .with_context(|| anyhow!("Parsing '{}' into a number", s))?,
                );
            }
            Command::InC => {
                let mut stdin = io::stdin();
                let mut buf = [0; 1];
                stdin.read_exact(&mut buf).context("Reading a byte")?;
                self.stack.push(buf[0].into());
            }
            Command::Bri => self.advance_pc(),
            Command::Space => {}
            Command::Num(n) => self.stack.push(n as StackTy),
            Command::Char(c) => self.stack.push(c.to_digit(10).unwrap().into()),
            Command::Get => {
                let y = self.stack.pop() as usize;
                let x = self.stack.pop() as usize;

                if x >= PLAYFIELD_COLS {
                    bail!("Invalid x coordinate for g command: {}", x);
                } else if y >= PLAYFIELD_ROWS {
                    bail!("Invalid y coordinate for g command: {}", y);
                }

                let cmd: char = self.playfield[y][x].into();
                self.stack.push((cmd as u8).into());
            }
            Command::Put => {
                let y = self.stack.pop() as usize;
                let x = self.stack.pop() as usize;

                if x >= PLAYFIELD_COLS {
                    bail!("Invalid x coordinate for p command: {}", x);
                } else if y >= PLAYFIELD_ROWS {
                    bail!("Invalid y coordinate for p command: {}", y);
                }

                let val = self.stack.pop();
                let val: u8 = val
                    .try_into()
                    .with_context(|| anyhow!("Failed to convert {} into u8", val))?;
                self.playfield[y][x] = Command::from(val as char);
            }
            Command::End => return Ok(StepResult::Stop),
        };

        self.advance_pc();
        Ok(StepResult::Cont)
    }

    fn advance_pc(&mut self) {
        match self.dir {
            Direction::Right => self.pc.right(),
            Direction::Left => self.pc.left(),
            Direction::Up => self.pc.up(),
            Direction::Down => self.pc.down(),
        }
    }

    pub fn run(&mut self, f: impl Fn(&Self, usize) -> bool) -> Result<()> {
        self.pc.reset();
        self.stack.reset();
        self.output.clear();

        let mut iter_n = 0;

        while self
            .step()
            .with_context(|| anyhow!("Stepping at {:?}", self.pc))?
            != StepResult::Stop
        {
            iter_n += 1;
            if !f(self, iter_n) {
                break;
            }
        }

        Ok(())
    }
}

impl ToString for Interpreter {
    fn to_string(&self) -> String {
        let mid_line = String::from("\u{2500}").repeat(PLAYFIELD_COLS);

        // Build top line
        let mut line = String::from("\u{250C}");
        line += &mid_line;
        line.push('\u{2510}');

        let mut s = Yellow.paint(&line).to_string();
        s.push('\n');

        for (row_idx, row) in self.playfield.iter().enumerate() {
            s += &Yellow.paint("\u{2502}").to_string();

            for (col_idx, cmd) in row.iter().enumerate() {
                // Highlight current PC
                if row_idx == self.pc.y && col_idx == self.pc.x {
                    s += &Red.on(White).bold().paint(cmd.to_string()).to_string();
                } else {
                    s.push(cmd.as_char());
                }
            }

            s += &Yellow.paint("\u{2502}\n").to_string();
        }

        // Build bottom line
        let mut line = String::from("\u{2514}");
        line += &mid_line;
        line.push('\u{2518}');

        s + &Yellow.paint(&line).to_string()
    }
}
