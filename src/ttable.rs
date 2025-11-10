use crate::{constants::Depth, Move, Score};
use std::{cmp::Reverse, mem};

pub struct TTable {
    buckets: Vec<Bucket>,
    epoch: u8,
}

impl TTable {
    pub fn new(size: usize) -> Self {
        let num_buckets = size / mem::size_of::<Bucket>();
        assert!(num_buckets > 0);
        let num_buckets = 1 << num_buckets.ilog2();
        Self {
            buckets: vec![Bucket::default(); num_buckets],
            epoch: 1,
        }
    }

    pub fn new_epoch(&mut self) {
        self.epoch = if self.epoch == u8::MAX {
            1
        } else {
            self.epoch + 1
        };
    }

    pub fn get(&mut self, hash: u64) -> Option<TTableEntry> {
        let (hash, bucket_idx) = self.split_hash(hash);
        let bucket = &mut self.buckets[bucket_idx];
        let entry = bucket
            .entries
            .iter_mut()
            .find(|bucket| bucket.hash == hash)?;
        entry.epoch = self.epoch;
        Some((&*entry).into())
    }

    pub fn set(&mut self, hash: u64, entry: TTableEntry) {
        let (hash, bucket_idx) = self.split_hash(hash);
        let bucket = &mut self.buckets[bucket_idx];
        let best_entry = bucket
            .entries
            .iter_mut()
            .max_by_key(|e| (e.hash == hash, e.epoch != self.epoch, Reverse(e.depth)))
            .unwrap();
        best_entry.hash = hash;
        best_entry.epoch = self.epoch;
        best_entry.depth = entry.depth;
        best_entry.mov = entry.mov;
        best_entry.score_type = entry.score_type;
        best_entry.score = entry.score;
    }

    fn split_hash(&self, hash: u64) -> (u32, usize) {
        (
            (hash >> 32) as u32,
            hash as usize & (self.buckets.len() - 1),
        )
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TTableEntry {
    pub depth: Depth,
    pub mov: Option<Move>,
    pub score_type: TTableScoreType,
    pub score: Score,
}

impl From<&PhysicalEntry> for TTableEntry {
    fn from(entry: &PhysicalEntry) -> Self {
        Self {
            depth: entry.depth,
            mov: entry.mov,
            score_type: entry.score_type,
            score: entry.score,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TTableScoreType {
    None,
    Exact,
    LowerBound,
    UpperBound,
}

impl Default for TTableScoreType {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Copy, Clone, Default)]
struct PhysicalEntry {
    hash: u32,
    epoch: u8,
    depth: Depth,
    mov: Option<Move>,
    score_type: TTableScoreType,
    score: Score,
}

const _: () = assert!(mem::size_of::<PhysicalEntry>() == 16);

#[derive(Debug, Copy, Clone, Default)]
#[repr(align(64))]
struct Bucket {
    entries: [PhysicalEntry; 4],
}

const _: () = assert!(mem::size_of::<Bucket>() == 64);
