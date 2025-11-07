use crate::LongVariation;
use std::mem;

pub struct PVTable {
    buckets: Vec<Bucket>,
    epoch: u8,
}

impl PVTable {
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

    pub fn get(&mut self, hash: u64) -> Option<LongVariation> {
        let (hash, bucket_idx) = self.split_hash(hash);
        let bucket = &mut self.buckets[bucket_idx];
        let (index, entry) = bucket
            .entries
            .iter_mut()
            .enumerate()
            .find(|(_, entry)| entry.hash == hash)?;
        entry.epoch = self.epoch;
        Some(bucket.variations[index].clone())
    }

    pub fn set(&mut self, hash: u64, variation: LongVariation) {
        let (hash, bucket_idx) = self.split_hash(hash);
        let bucket = &mut self.buckets[bucket_idx];
        let (index, best_entry) = bucket
            .entries
            .iter_mut()
            .enumerate()
            .max_by_key(|(_, entry)| (entry.hash == hash, entry.epoch != self.epoch))
            .unwrap();
        *best_entry = Entry {
            hash,
            epoch: self.epoch,
        };
        bucket.variations[index] = variation;
    }

    fn split_hash(&self, hash: u64) -> (u32, usize) {
        (
            (hash >> 32) as u32,
            hash as usize & (self.buckets.len() - 1),
        )
    }
}

#[derive(Debug, Copy, Clone, Default)]
struct Entry {
    hash: u32,
    epoch: u8,
}

#[derive(Default, Clone)]
#[repr(align(64))]
struct Bucket {
    entries: [Entry; Self::SIZE],
    variations: [LongVariation; Self::SIZE],
}

impl Bucket {
    const SIZE: usize = 4;
}
