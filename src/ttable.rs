use crate::{RegularMove, Score};
use std::{cmp::Reverse, mem};

pub struct TTable {
    buckets: Vec<Bucket>,
    epoch: u8,
}

impl TTable {
    pub fn new(size: usize) -> Self {
        let num_buckets = 1 << (size / mem::size_of::<Bucket>()).ilog2();
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

    pub fn get(&self, hash: u64) -> Option<TTableEntry> {
        let (hash, bucket_idx) = self.split_hash(hash);
        let bucket = &self.buckets[bucket_idx];
        let entry = bucket.entries.iter().find(|bucket| bucket.hash == hash)?;
        Some(TTableEntry {
            depth: entry.depth,
            mov: entry.mov,
            score: TTableScore::combine(entry.score_type, entry.score),
        })
    }

    pub fn set(&mut self, hash: u64, entry: TTableEntry) {
        let (hash, bucket_idx) = self.split_hash(hash);
        let bucket = &mut self.buckets[bucket_idx];
        let best_entry = bucket
            .entries
            .iter_mut()
            .max_by_key(|entry| {
                (
                    entry.hash == hash,
                    entry.epoch != self.epoch,
                    Reverse(entry.depth),
                )
            })
            .unwrap();
        best_entry.hash = hash;
        best_entry.depth = entry.depth;
        best_entry.mov = entry.mov;
        (best_entry.score_type, best_entry.score) = entry.score.split();
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
    pub depth: u16,
    pub mov: Option<RegularMove>,
    pub score: TTableScore,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TTableScore {
    None,
    Exact(Score),
    LowerBound(Score),
    UpperBound(Score),
}

impl TTableScore {
    fn combine(score_type: ScoreType, score: Score) -> Self {
        match score_type {
            ScoreType::None => Self::None,
            ScoreType::Exact => Self::Exact(score),
            ScoreType::LowerBound => Self::LowerBound(score),
            ScoreType::UpperBound => Self::UpperBound(score),
        }
    }

    fn split(self) -> (ScoreType, Score) {
        match self {
            Self::None => (ScoreType::None, Score::default()),
            Self::Exact(score) => (ScoreType::Exact, score),
            Self::LowerBound(score) => (ScoreType::LowerBound, score),
            Self::UpperBound(score) => (ScoreType::UpperBound, score),
        }
    }
}

impl Default for TTableScore {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Copy, Clone)]
enum ScoreType {
    None,
    Exact,
    LowerBound,
    UpperBound,
}

impl Default for ScoreType {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Copy, Clone, Default)]
struct PhysicalEntry {
    hash: u32,
    depth: u16,
    mov: Option<RegularMove>,
    score_type: ScoreType,
    score: Score,
    epoch: u8,
}

const _: () = assert!(mem::size_of::<PhysicalEntry>() == 16);

#[derive(Debug, Copy, Clone, Default)]
#[repr(align(64))]
struct Bucket {
    entries: [PhysicalEntry; 4],
}

const _: () = assert!(mem::size_of::<Bucket>() == 64);
