use std::collections::HashMap;

use metrohash::MetroBuildHasher;

use crate::{pos, Cell, Pos, World};

const CHUNK_SIZE: usize = 16;

#[derive(Debug, Default, Clone)]
struct Chunk {
    cells: [[Cell; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Chunk {
    fn get(&self, pos: Pos) -> Cell {
        let pos = HashedWorld::get_local_pos(pos);
        self.cells[pos.x as usize][pos.y as usize].clone()
    }

    fn set(&mut self, pos: Pos, cell: Cell) {
        let pos = HashedWorld::get_local_pos(pos);
        self.cells[pos.x as usize][pos.y as usize] = cell;
    }

    fn get_actives(&self) -> impl Iterator<Item = Pos> + '_ {
        self.cells.iter().enumerate().flat_map(|(x, row)| {
            row.iter()
                .enumerate()
                .filter_map(move |(y, cell)| cell.is_active().then_some(pos!(x as i32, y as i32)))
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ChunkPos(Pos);

#[derive(Debug, Clone, Default)]
pub struct HashedWorld {
    chunks: HashMap<ChunkPos, Chunk, MetroBuildHasher>,
}

impl HashedWorld {
    /// gets the position of a chunk containing the passed position
    fn get_chunk_pos(Pos { x, y }: Pos) -> ChunkPos {
        let x = snap(x, CHUNK_SIZE as i32);
        let y = snap(y, CHUNK_SIZE as i32);
        ChunkPos(pos!(x, y))
    }

    /// gets the position of a cell local to it's parent chunk.
    fn get_local_pos(pos: Pos) -> Pos {
        let ChunkPos(chunk_pos) = Self::get_chunk_pos(pos);
        pos - chunk_pos
    }

    fn get_chunk(&self, pos: Pos) -> Option<&Chunk> {
        let pos = Self::get_chunk_pos(pos);
        self.chunks.get(&pos)
    }

    fn get_chunk_mut(&mut self, pos: Pos) -> Option<&mut Chunk> {
        let pos = Self::get_chunk_pos(pos);
        self.chunks.get_mut(&pos)
    }

    fn push_chunk(&mut self, pos: Pos, chunk: Chunk) {
        let chunk_pos = Self::get_chunk_pos(pos);
        self.chunks.insert(chunk_pos, chunk);
    }
}

pub fn snap(n: i32, step: i32) -> i32 {
    // frankly, I forgot how it works, but somehow it passes tests
    let rem = ((n % step) + step) % step;
    n - rem
}

#[test]
fn test_snap() {
    assert_eq!(snap(0, 10), 0);
    assert_eq!(snap(1, 10), 0);
    assert_eq!(snap(-1, 10), -10);
    assert_eq!(snap(10, 10), 10);
    assert_eq!(snap(11, 10), 10);
}

impl World for HashedWorld {
    fn get(&self, pos: Pos) -> Cell {
        if let Some(chunk) = self.get_chunk(pos) {
            chunk.get(pos)
        } else {
            Cell::inactive()
        }
    }

    fn set(&mut self, pos: Pos, cell: Cell) {
        if let Some(chunk) = self.get_chunk_mut(pos) {
            chunk.set(pos, cell)
        } else {
            let mut chunk = Chunk::default();
            chunk.set(pos, cell);
            self.push_chunk(pos, chunk)
        }
    }

    fn actives(&self) -> Vec<Pos> {
        self.chunks
            .iter()
            .flat_map(|(ChunkPos(chunk_pos), chunk)| {
                chunk.get_actives().map(|pos| *chunk_pos + pos)
            })
            .collect()
    }

    fn dbg_is_loaded(&self, pos: Pos) -> bool {
        self.get_chunk(pos).is_some()
    }
}
