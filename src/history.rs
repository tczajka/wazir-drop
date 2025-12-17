use crate::{
    constants::{Ply, HISTORY_BLOOM_FILTER_LOG_SIZE, HISTORY_BLOOM_FILTER_NUM_HASHES},
    Position,
};
use std::iter;

#[derive(Clone, Debug)]
pub struct History {
    irreversible: Vec<Ply>, // ply after an irreversible move
    hashes: Vec<u64>,
    bloom_filter: Vec<u8>,
}

impl History {
    pub fn new(hash: u64) -> Self {
        Self {
            irreversible: vec![0],
            hashes: vec![hash],
            bloom_filter: vec![0; 1 << HISTORY_BLOOM_FILTER_LOG_SIZE],
        }
    }

    pub fn new_from_position(position: &Position) -> Self {
        Self::new(position.hash_for_repetition())
    }

    pub fn ply(&self) -> Ply {
        (self.hashes.len() - 1) as Ply
    }

    pub fn push(&mut self, hash: u64) {
        self.hashes.push(hash);
        for index in Self::indices(hash) {
            self.bloom_filter[index] += 1;
        }
    }

    pub fn push_irreversible(&mut self, hash: u64) {
        self.irreversible.push(self.hashes.len() as Ply);
        self.push(hash);
    }

    pub fn push_position(&mut self, position: &Position) {
        self.push(position.hash_for_repetition());
    }

    pub fn push_position_irreversible(&mut self, position: &Position) {
        self.push_irreversible(position.hash_for_repetition());
    }

    pub fn pop(&mut self) {
        let hash = self.hashes.pop().unwrap();
        for index in Self::indices(hash) {
            self.bloom_filter[index] -= 1;
        }
        assert!(!self.hashes.is_empty());
        if *self.irreversible.last().unwrap() == self.hashes.len() as Ply {
            _ = self.irreversible.pop();
        }
    }

    pub fn find_repetition(&self) -> Option<Ply> {
        let mut ply = self.hashes.len() - 1;
        let hash = self.hashes[ply];
        for index in Self::indices(hash) {
            if self.bloom_filter[index] < 2 {
                return None;
            }
        }
        let start = *self.irreversible.last().unwrap() as usize;
        while ply >= start + 2 {
            ply -= 2;
            if self.hashes[ply] == hash {
                return Some(ply as Ply);
            }
        }
        None
    }

    pub fn last_move_irreversible(&self) -> bool {
        *self.irreversible.last().unwrap() as usize == self.hashes.len() - 1
    }

    fn indices(mut hash: u64) -> impl Iterator<Item = usize> {
        const MASK: usize = (1 << HISTORY_BLOOM_FILTER_LOG_SIZE) - 1;
        iter::repeat_with(move || {
            let index = hash as usize & MASK;
            hash >>= HISTORY_BLOOM_FILTER_LOG_SIZE;
            index
        })
        .take(HISTORY_BLOOM_FILTER_NUM_HASHES)
    }
}
