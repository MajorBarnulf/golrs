use std::{env::args, fs, process::exit};

pub use utils::Pos;
mod utils;

pub use world::{Cell, HashedWorld, World};
pub mod world;

pub use sim::{Sim, SimHandle};
mod sim;

pub use view::View;
mod view;

fn deserialize(str: &str) -> Vec<Pos> {
    let mut result = vec![];
    let mut pos = pos!(0, 0);
    for c in str.chars() {
        match c {
            '#' => {
                result.push(pos);
                pos.x += 1
            }
            '\n' => pos = pos!(0, pos.y + 1),
            _ => pos.x += 1,
        }
    }
    result
}

pub fn main() {
    let path = args().nth(1).unwrap_or_else(|| {
        eprintln!("[error] must provide a path argument");
        exit(1);
    });

    let content = fs::read_to_string(path).unwrap();
    let actives = deserialize(&content);
    let simulation = Sim::spawn(actives);
    let view = View::spawn::<HashedWorld>(simulation.handle());

    simulation.join();
    view.join();
}
