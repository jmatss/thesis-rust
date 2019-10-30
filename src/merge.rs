use md5::Digest;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::error::Error;
use std::sync::{mpsc, Arc};
use std::sync::mpsc::{SyncSender, Receiver, RecvError};
use std::thread;
use crate::errors::GeneralError;
use crate::block::Block;
use crate::cons::START_CMP;
use std::any::Any;

struct DigestWithID {
    pub id: usize,
    pub digest: Digest,
}

impl DigestWithID {
    fn new(id: usize, digest: Digest) -> Self {
        DigestWithID { id, digest }
    }
}

impl PartialOrd for DigestWithID {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.digest[START_CMP..].cmp(&other.digest[START_CMP..]))
    }
}

impl std::cmp::Ord for DigestWithID {
    fn cmp(&self, other: &Self) -> Ordering {
        self.digest[START_CMP..].cmp(&other.digest[START_CMP..])
    }
}

impl Eq for DigestWithID {}

impl PartialEq for DigestWithID {
    fn eq(&self, other: &Self) -> bool {
        self.digest == other.digest
    }
}

pub fn merge_blocks(
    blocks: Vec<Block>,
    amount_of_threads: usize,
    mut buffer_size: u64,
    filename: String,
    print_amount: u64,
) -> Result<(), Box<dyn Error>> {
    const CHANNEL_BUFFER_SIZE: usize = 10;

    if amount_of_threads == 0 {
        return Err(Box::new(GeneralError::new(
            String::from("Amount of threads set to zero.")
        )));
    }

    let (sender, receiver) = mpsc::sync_channel::<Digest>(
        CHANNEL_BUFFER_SIZE
    );

    let t = thread::spawn(|| {
        merge_handler(blocks, amount_of_threads, buffer_size, filename, sender)
    });

    Ok(())
}

fn merge_handler(
    mut blocks: Vec<Block>,
    mut amount_of_threads: usize,
    mut buffer_size: u64,
    filename: String,
    sender: SyncSender<Digest>,
) -> Result<(), Box<dyn Any + Send>> {
    let a = Arc::new(blocks);
    let buffer_size_per_block = buffer_size / blocks.len() as u64;
    for ref mut block in blocks {
        block.init_merge(buffer_size_per_block);
    }

    let mut blocks_per_thread = blocks.len() / amount_of_threads;
    if blocks.len() < amount_of_threads {
        amount_of_threads = blocks.len();
        blocks_per_thread = 1;
    }

    let mut thread_channels: Vec<Receiver<Digest>> = Vec::with_capacity(amount_of_threads);
    let mut priority_queue: BinaryHeap<DigestWithID> = BinaryHeap::with_capacity(amount_of_threads);
    let mut remaining = blocks.as_mut_slice();

    crossbeam_utils::thread::scope(|s| -> Result<(), Box<dyn Any + Send>> {
        for i in 0..amount_of_threads {
            if i == amount_of_threads - 1 {
                blocks_per_thread = remaining.len();
            }

            let (current, rest) = remaining.split_at_mut(blocks_per_thread);
            remaining = rest;

            // TODO: Fix buffer size const.
            let (tx, rx) = mpsc::sync_channel(10);
            thread_channels.push(rx);

            s.spawn(move || merge_handler_thread(current, tx));

            let r = thread_channels[i].recv()?;
            let digest_with_id = DigestWithID::new(i,r );
            priority_queue.push(digest_with_id);
        }

        Ok(())
    })?
}

fn merge_handler_thread(blocks: &[Block], sender: SyncSender<Digest>) -> Result<(), Box<dyn Any + Send>> {
    Ok(())
}