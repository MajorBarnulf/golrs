use crate::Pos;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Cell {
    active: bool,
}

impl Cell {
    pub fn active() -> Self {
        Self { active: true }
    }

    pub fn inactive() -> Self {
        Self { active: false }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
}

pub trait World: Default + Clone + Send + 'static {
    fn get(&self, pos: Pos) -> Cell;
    fn set(&mut self, pos: Pos, cell: Cell);
    fn actives(&self) -> Vec<Pos>;
    fn dbg_is_loaded(&self, pos: Pos) -> bool;
}

pub use hashed_world::HashedWorld;
mod hashed_world;
