use std::{
    sync::mpsc,
    thread::{self, JoinHandle},
    time::{Duration, SystemTime},
};

use crate::{pos, Cell, Pos, World};

#[derive(Debug, Default)]
pub struct State<W>
where
    W: World,
{
    world: W,
}

impl<W> State<W>
where
    W: World,
{
    pub fn actives(&self) -> Vec<Pos> {
        self.world.actives()
    }

    pub fn get(&self, pos: Pos) -> Cell {
        self.world.get(pos)
    }

    pub fn set(&mut self, pos: Pos, cell: Cell) {
        self.world.set(pos, cell)
    }

    fn possible_change_pos(&self) -> impl Iterator<Item = Pos> + '_ {
        self.actives()
            .into_iter()
            .map(|p| self.get_neighbors(p))
            .flatten()
    }

    pub fn get_neighbors(&self, pos: Pos) -> impl Iterator<Item = Pos> + '_ {
        (-1..=1)
            .map(|x| (-1..=1).map(move |y| pos!(x, y)))
            .flatten()
            .map(move |p| pos + p)
    }

    pub fn is_cell_alive(&self, pos: Pos) -> bool {
        self.get(pos.clone()).is_active()
    }

    pub fn get_neighbor_count(&self, pos: Pos) -> usize {
        self.get_neighbors(pos)
            .filter(|pos| self.is_cell_alive(pos.clone()))
            .count()
    }

    pub fn snapshot(&self) -> W {
        self.world.clone()
    }
}

pub enum SimCmd<W>
where
    W: World,
{
    Snapshot(mpsc::Sender<W>),
}

pub struct SimHandle<W>
where
    W: World,
{
    sender: mpsc::Sender<SimCmd<W>>,
}

impl<W> SimHandle<W>
where
    W: World,
{
    pub fn new(sender: mpsc::Sender<SimCmd<W>>) -> Self {
        Self { sender }
    }

    pub fn snapshot(&self) -> W {
        let (sender, receiver) = mpsc::channel();
        self.sender.send(SimCmd::Snapshot(sender)).unwrap();
        receiver.recv().unwrap()
    }
}

#[derive(Debug)]
pub struct Sim<W>
where
    W: World,
{
    thread: JoinHandle<()>,
    sender: mpsc::Sender<SimCmd<W>>,
}

impl<W> Sim<W>
where
    W: World,
{
    pub fn spawn(actives: impl IntoIterator<Item = Pos>) -> Self {
        let mut state: State<W> = State::default();
        for active in actives.into_iter() {
            state.set(active, Cell::active());
        }

        let (sender, receiver) = mpsc::channel();
        let thread = thread::spawn(move || sim_loop(receiver, state));

        Self { sender, thread }
    }

    pub fn handle(&self) -> SimHandle<W> {
        let sender = self.sender.clone();
        SimHandle { sender }
    }

    pub fn join(self) {
        self.thread.join().unwrap();
    }
}

const EVT_CHECK_TIMEOUT: Duration = Duration::from_millis(10);
const SIM_TICK_INTERVAL: Duration = Duration::from_millis(200);

fn sim_loop<W>(receiver: mpsc::Receiver<SimCmd<W>>, state: State<W>)
where
    W: World,
{
    let mut current_state = state;
    let mut last_update = SystemTime::now();

    loop {
        if let Some(cmd) = receiver.try_recv().ok() {
            match cmd {
                SimCmd::Snapshot(sender) => sender.send(current_state.snapshot()).unwrap(),
            }
        }

        if SystemTime::now().duration_since(last_update).unwrap() > SIM_TICK_INTERVAL {
            let old_state = current_state;
            let mut new_state: State<W> = State::default();

            for pos in old_state.possible_change_pos() {
                let is_active = old_state.is_cell_alive(pos);
                let neighbor_count = old_state.get_neighbor_count(pos);
                match (is_active, neighbor_count) {
                    (true, count) if count < 3 || count > 4 => (),   // die
                    (true, _) => new_state.set(pos, Cell::active()), // stay
                    (false, 3) => new_state.set(pos, Cell::active()), // becomes alive
                    _ => (),                                         // stays dead
                }
            }
            current_state = new_state;
            last_update = SystemTime::now();
        }

        thread::sleep(EVT_CHECK_TIMEOUT);
    }
}
