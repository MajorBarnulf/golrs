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

pub struct View<W>
where
	W: World,
{
	thread: JoinHandle<()>,
	sender: mpsc::Sender<ViewCmd<W>>,
}
impl<W> View<W>
where
	W: World,
{
	pub fn spawn(handle: SimHandle<W>) -> Self
	where
		W: World,
	{
		let (sender, receiver) = mpsc::channel();
		let thread = thread::spawn(|| view_loop(receiver, handle));
		Self { thread, sender }
	}

	pub fn join(self) {
		self.thread.join().unwrap();
	}
}

pub struct ViewRemote<W>
where
	W: World,
{
	sender: mpsc::Sender<ViewCmd<W>>,
}

pub enum ViewCmd<W>
where
	W: World,
{
	Refresh,
	UpdateWorld(W),
}

impl<W> ViewRemote<W>
where
	W: World,
{
	fn new(sender: mpsc::Sender<ViewCmd<W>>) -> Self {
		Self { sender }
	}
	pub fn refresh(&self) {
		self.sender.send(ViewCmd::Refresh).unwrap();
	}

	pub fn set_world(&self, world: W) {
		self.sender.send(ViewCmd::UpdateWorld(world)).unwrap();
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
	ToggleDebug,
	Move(Dir),
	Accelerate,
	Decelerate,
}

fn input_loop(sender: mpsc::Sender<InputCmd>) {
	let stdout = stdout().into_raw_mode().unwrap();
	for c in stdin().keys() {
		let command = match c.unwrap() {
			Key::Char('q') => InputCmd::Exit,
			Key::Char('d') => InputCmd::ToggleDebug,
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
fn view_loop<W>(receiver: mpsc::Receiver<ViewCmd<W>>, handle: SimHandle<W>)
where
	W: World,
{
	let (sender, receiver) = mpsc::channel();
	let _input_handle = thread::spawn(|| input_loop(sender));

	let (x, y) = termion::terminal_size().unwrap();
	let mut tick_delay = 200;
	let mut view_origin = pos!(-(x as i32) / 2, -(y as i32));
	let mut debug = false;
	loop {
		handle_inputs(&receiver, &mut view_origin, &mut debug, &mut tick_delay);
		handle.set_delay(tick_delay);
		let world = handle.snapshot();

		let mut canvas = Canvas::from_screen();
		if debug {
			dbg_layer(&mut canvas, &world, view_origin);
		}
		grid_layer(&mut canvas, view_origin);
		world_layer(&mut canvas, &world, view_origin);
		title_layer(&mut canvas, handle.delay());
		canvas.display();
	}
	drop(handle);
}

const MOVEMENT_STEP: i32 = 3;

fn handle_inputs(
	receiver: &mpsc::Receiver<InputCmd>,
	view_origin: &mut Pos,
	debug: &mut bool,
	delay: &mut u64,
) {
	if let Ok(cmd) = receiver.recv_timeout(VIEW_REFRESH_INTERVAL) {
		match cmd {
			InputCmd::Exit => {
				println!("{}{}", termion::clear::All, termion::cursor::Goto(1, 1));
				exit(0);
			}
			InputCmd::ToggleDebug => *debug = !*debug,
			InputCmd::Move(direction) => {
				*view_origin = *view_origin
					+ match direction {
						Dir::Up => pos!(0, -MOVEMENT_STEP),
						Dir::Down => pos!(0, MOVEMENT_STEP),
						Dir::Left => pos!(-2 * MOVEMENT_STEP, 0),
						Dir::Right => pos!(2 * MOVEMENT_STEP, 0),
					}
			}
			InputCmd::Accelerate => {
				if *delay > 0 {
					*delay -= 100
				}
			}
			InputCmd::Decelerate => *delay += 100,
		}
	}
}

fn grid_layer(canvas: &mut Canvas, view_origin: Pos) {
	canvas.layer(|local_pos| {
		let Pos { x, y } = screen_pos_to_world(local_pos, view_origin);
		match (x % 16 == 0, dmod(y, 16) <= 1) {
			(true, true) => Some('┼'),
			(true, _) => Some('│'),
			(_, true) => Some('─'),
			_ => None,
		}
	})
}

fn world_layer<W>(canvas: &mut Canvas, world: &W, view_origin: Pos)
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
		format!("│             | [-]: slow down │"),
		format!("│ tick delay: | [d]: debug     │"),
		format!("│  {de:>7} ms | [q]: quit      │"),
		format!("└──────────────────────────────┘"),
	];
	canvas.layer(|Pos { x, y }| {
		table
			.get(y as usize)
			.and_then(|line| line.chars().nth(x as usize))
	})
}

fn dbg_layer<W>(canvas: &mut Canvas, world: &W, view_origin: Pos)
where
	W: World,
{
	canvas.layer(|local_pos| {
		let pos = screen_pos_to_world(local_pos, view_origin);
		world.dbg_is_loaded(pos).then_some('.')
	})
}

fn screen_pos_to_world(mut pos: Pos, view_origin: Pos) -> Pos {
	pos.y *= 2;
	pos + view_origin
}

/// double module to avoid irregularities between negatives and positives
fn dmod(a: i32, module: i32) -> i32 {
	((a % module) + module) % module
}
