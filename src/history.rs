use std::iter;

use crate::constants::{
    Ply, HISTORY_BLOOM_FILTER_LOG_SIZE, HISTORY_BLOOM_FILTER_NUM_HASHES, PLY_DRAW,
};

pub struct History {
    root_ply: Ply,
    hashes: Vec<u64>,
    bloom_filter: Vec<u8>,
}

impl History {
    pub fn new() -> Self {
        Self {
            root_ply: 0,
            hashes: Vec::with_capacity(PLY_DRAW.into()),
            bloom_filter: vec![0; 1 << HISTORY_BLOOM_FILTER_LOG_SIZE],
        }
    }

    pub fn clear(&mut self, root_ply: Ply) {
        self.root_ply = root_ply;
        self.hashes.clear();
        self.bloom_filter.fill(0);
    }

    pub fn push(&mut self, hash: u64) {
        self.hashes.push(hash);
        for index in Self::indices(hash) {
            self.bloom_filter[index] += 1;
        }
    }

    pub fn pop(&mut self) {
        let hash = self.hashes.pop().unwrap();
        for index in Self::indices(hash) {
            self.bloom_filter[index] -= 1;
        }
    }

    pub fn find(&self, hash: u64) -> Option<Ply> {
        for index in Self::indices(hash) {
            if self.bloom_filter[index] == 0 {
                return None;
            }
        }
        self.hashes
            .iter()
            .copied()
            .enumerate()
            .rev()
            .skip(1)
            .step_by(2)
            .find(|&(_, h)| h == hash)
            .map(|(ply, _)| self.root_ply + ply as Ply)
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
