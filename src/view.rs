use std::{
    io::{stdin, stdout, Write},
    process::exit,
    sync::mpsc,
    thread::{self, JoinHandle},
    time::Duration,
};

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
            _ => continue,
        };

        sender.send(command).unwrap();
    }
    drop(stdout);
}

const VIEW_REFRESH_INTERVAL: Duration = Duration::from_millis(100);

fn view_loop<W>(handle: SimHandle<W>)
where
    W: World,
{
    let (sender, receiver) = mpsc::channel();
    let _input_handle = thread::spawn(|| input_loop(sender));

    let mut view_origin = pos!(0, 0);
    loop {
        handle_inputs(&receiver, &mut view_origin);
        let world = handle.snapshot();
        display_world(view_origin, world);
        thread::sleep(VIEW_REFRESH_INTERVAL);
    }
    drop(handle);
}

fn handle_inputs(receiver: &mpsc::Receiver<InputCmd>, view_origin: &mut Pos) {
    if let Some(cmd) = receiver.try_recv().ok() {
        match cmd {
            InputCmd::Exit => exit(0),
            InputCmd::Move(direction) => {
                *view_origin = *view_origin
                    + match direction {
                        Dir::Up => pos!(0, -4),
                        Dir::Down => pos!(0, 4),
                        Dir::Left => pos!(-4, 0),
                        Dir::Right => pos!(4, 0),
                    }
            }
            InputCmd::Accelerate => todo!(),
            InputCmd::Decelerate => todo!(),
        }
    }
}

fn display_world<W>(view_origin: Pos, world: W)
where
    W: World,
{
    let (width, height) = termion::terminal_size().unwrap();
    let mut result = String::new();

    for ly in 0..(height) {
        let next_line = termion::cursor::Goto(1, ly + 1);
        result += &format!("{next_line}");
        for lx in 0..width {
            let pos = view_origin + pos!(lx as i32, ly as i32);
            let char = &if world.get(pos).is_active() { "#" } else { " " };
            result += char
        }
    }
    let clear = termion::clear::All;
    print!("{clear}{result}");
    stdout().flush().unwrap();
}
