use std::{
    io::{stdin, stdout},
    process::exit,
    sync::mpsc,
    thread::{self, JoinHandle},
    time::Duration,
};

pub use canvas::Canvas;
mod canvas;

use termion::{event::Key, input::TermRead, raw::IntoRawMode};

use crate::{pos, Pos, SimHandle, World};

pub struct View {
    thread: JoinHandle<()>,
}
impl View {
    pub fn spawn<W>(handle: SimHandle<W>) -> Self
    where
        W: World,
    {
        let thread = thread::spawn(|| view_loop(handle));
        Self { thread }
    }

    pub fn join(self) {
        self.thread.join().unwrap();
    }
}

#[derive(Debug)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
pub enum InputCmd {
    Exit,
    Move(Dir),
    Accelerate,
    Decelerate,
}

fn input_loop(sender: mpsc::Sender<InputCmd>) {
    let stdout = stdout().into_raw_mode().unwrap();
    for c in stdin().keys() {
        let command = match c.unwrap() {
            Key::Char('q') => InputCmd::Exit,
            Key::Up => InputCmd::Move(Dir::Up),
            Key::Down => InputCmd::Move(Dir::Down),
            Key::Left => InputCmd::Move(Dir::Left),
            Key::Right => InputCmd::Move(Dir::Right),
            Key::Char('+') => InputCmd::Accelerate,
            Key::Char('-') => InputCmd::Decelerate,
            _ => continue,
        };

        sender.send(command).unwrap();
    }
    drop(stdout);
}

const VIEW_REFRESH_INTERVAL: Duration = Duration::from_millis(100);

#[allow(unreachable_code)]
fn view_loop<W>(handle: SimHandle<W>)
where
    W: World,
{
    let (sender, receiver) = mpsc::channel();
    let _input_handle = thread::spawn(|| input_loop(sender));

    let (x, y) = termion::terminal_size().unwrap();
    let mut delay = 200u64;
    let mut view_origin = pos!(-(x as i32) / 2, -(y as i32));
    loop {
        handle_inputs(&receiver, &mut view_origin, &mut delay);
        handle.set_delay(delay);

        let mut canvas = Canvas::from_screen();
        grid_layer(&mut canvas, view_origin);
        world_layer(&mut canvas, handle.snapshot(), view_origin);
        title_layer(&mut canvas, handle.delay());
        canvas.display();
    }
    drop(handle);
}

const MOVEMENT_STEP: i32 = 3;

fn handle_inputs(receiver: &mpsc::Receiver<InputCmd>, view_origin: &mut Pos, delay: &mut u64) {
    if let Ok(cmd) = receiver.recv_timeout(VIEW_REFRESH_INTERVAL) {
        match cmd {
            InputCmd::Exit => {
                println!("{}{}", termion::clear::All, termion::cursor::Goto(1, 1));
                exit(0);
            }
            InputCmd::Move(direction) => {
                *view_origin = *view_origin
                    + match direction {
                        Dir::Up => pos!(0, -MOVEMENT_STEP),
                        Dir::Down => pos!(0, MOVEMENT_STEP),
                        Dir::Left => pos!(-2 * MOVEMENT_STEP, 0),
                        Dir::Right => pos!(2 * MOVEMENT_STEP, 0),
                    }
            }
            InputCmd::Accelerate => *delay -= 100,
            InputCmd::Decelerate => *delay += 100,
        }
    }
}

fn grid_layer(canvas: &mut Canvas, view_origin: Pos) {
    canvas.layer(|local_pos| {
        let Pos { x, y } = local_pos + view_origin;
        match (x % 16 == 0, y % 8 == 0) {
            (true, true) => Some('┼'),
            (true, _) => Some('│'),
            (_, true) => Some('─'),
            _ => None,
        }
    })
}

fn world_layer<W>(canvas: &mut Canvas, world: W, view_origin: Pos)
where
    W: World,
{
    canvas.layer(|mut local_pos| {
        local_pos.y *= 2;
        let pos_top = local_pos + view_origin;
        let top = world.get(pos_top).is_active();
        let pos_bottom = pos_top + pos!(0, 1);
        let bottom = world.get(pos_bottom).is_active();

        match (top, bottom) {
            (true, true) => Some('█'),
            (true, _) => Some('▀'),
            (_, true) => Some('▄'),
            _ => None,
        }
    });
}

#[allow(clippy::useless_format)]
fn title_layer(canvas: &mut Canvas, de: usize) {
    let table = [
        format!("│   <golrs>   | [+]: speed up  │"),
        format!("│ delay:      | [-]: slow down │"),
        format!("│  {de:>7} ms | [q]: quit      │"),
        format!("└──────────────────────────────┘"),
    ];
    canvas.layer(|Pos { x, y }| {
        table
            .get(y as usize)
            .and_then(|line| line.chars().nth(x as usize))
    })
}
