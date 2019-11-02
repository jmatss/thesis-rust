use crate::block::Block;
use crate::cons::{BUF_SIZE, START_CMP};
use crate::errors::GeneralError;
use crossbeam_utils::thread as cb_thread;
use md5::Digest;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, SyncSender};
use std::thread;

#[derive(Debug)]
struct DigestWithID {
    pub id: usize,
    pub digest: Option<Digest>,
}

impl DigestWithID {
    fn new(id: usize, digest: Option<Digest>) -> Self {
        DigestWithID { id, digest }
    }
}

impl PartialOrd for DigestWithID {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.digest?[START_CMP..].cmp(&other.digest?[START_CMP..]))
    }
}

/// Treats a "None"-option as greater than.
impl std::cmp::Ord for DigestWithID {
    fn cmp(&self, other: &Self) -> Ordering {
        if let Some(s) = self.digest {
            if let Some(o) = other.digest {
                s[START_CMP..].cmp(&o[START_CMP..])
            } else {
                Ordering::Less
            }
        } else {
            Ordering::Greater
        }
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
    buffer_size: u64,
    filename: &str,
    print_amount: u64,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    const CHANNEL_BUFFER_SIZE: usize = 10;

    if amount_of_threads == 0 {
        return Err(Box::new(GeneralError::new(String::from(
            "Amount of threads set to zero.",
        ))));
    }

    let mut file_writer = BufWriter::with_capacity(BUF_SIZE, File::create(filename)?);
    let (tx_child, rx_child) = mpsc::sync_channel::<Option<Digest>>(CHANNEL_BUFFER_SIZE);
    let merge_handler = thread::spawn(move || -> Result<(), Box<dyn Error + Send + Sync>> {
        merge_handler(blocks, amount_of_threads, buffer_size, tx_child)
    });

    let mut count: u64 = 0;
    while let Some(min) = rx_child.recv().expect("ABAAB") {
        file_writer.write_all(min.as_ref())?;

        count += 1;
        if count % print_amount == 0 {
            println!("{} hashes merged.", count);
        }
    }

    // TODO: Fix weird error handling.
    file_writer.flush()?;
    merge_handler
        .join()
        .map_err(|_| Box::new(GeneralError::new(String::from("a"))))?
}

fn merge_handler(
    mut blocks: Vec<Block>,
    mut amount_of_threads: usize,
    buffer_size: u64,
    tx_parent: SyncSender<Option<Digest>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let buffer_size_per_block = buffer_size / blocks.len() as u64;
    for block in blocks.iter_mut() {
        block
            .init_merge(buffer_size_per_block)
            .expect("Init merge failed.");
    }

    let mut blocks_per_thread = if blocks.len() < amount_of_threads {
        amount_of_threads = blocks.len();
        1
    } else {
        blocks.len() / amount_of_threads
    };

    let mut rx_channels: Vec<Receiver<Option<Digest>>> = Vec::with_capacity(amount_of_threads);
    let mut priority_queue: BinaryHeap<DigestWithID> = BinaryHeap::with_capacity(amount_of_threads);
    let mut remaining = blocks.as_mut_slice();

    cb_thread::scope(|s| {
        for i in 0..amount_of_threads {
            if i == amount_of_threads - 1 {
                blocks_per_thread = remaining.len();
            }

            let (current, rest) = remaining.split_at_mut(blocks_per_thread);
            remaining = rest;

            // TODO: Fix buffer size const.
            let (tx_child, rx_child) = mpsc::sync_channel::<Option<Digest>>(10);
            rx_channels.push(rx_child);

            s.spawn(move |_| merge_handler_thread(current, &tx_child));

            let digest_with_id = DigestWithID::new(i, rx_channels[i].recv()?);
            priority_queue.push(digest_with_id);
        }

        loop {
            if priority_queue.is_empty() {
                // Indicate to parent that it is done by sending None.
                tx_parent.send(None)?;
                break;
            }

            // Var "next" = Next minimum from same thread as "min".
            if let Some(min) = priority_queue.pop() {
                if let Some(next) = rx_channels[min.id].recv()? {
                    priority_queue.push(DigestWithID::new(min.id, Some(next)));
                } else {
                    // TODO: Close channel(?)
                }
                tx_parent.send(min.digest)?;
            } else {
                // TODO: error, pq should never contain None.
            }
        }

        Ok(())
    })
    .map_err(|_| Box::new(GeneralError::new(String::from("a"))))?
    // TODO: Fix weird error handling.
}

fn merge_handler_thread(
    sub_blocks: &mut [Block],
    tx_parent: &SyncSender<Option<Digest>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut priority_queue: BinaryHeap<DigestWithID> = BinaryHeap::with_capacity(sub_blocks.len());

    for (i, block) in sub_blocks.iter_mut().enumerate() {
        priority_queue.push(DigestWithID::new(i, block.pop()));
    }

    loop {
        if priority_queue.is_empty() {
            tx_parent.send(None)?;
            break;
        }

        // Var "next" = Next minimum from same block as "min".
        if let Some(min) = priority_queue.pop() {
            if let Some(next) = sub_blocks[min.id].pop() {
                priority_queue.push(DigestWithID::new(min.id, Some(next)));
            }
            tx_parent.send(min.digest)?;
        } else {
            // TODO: throw error, pq should never contain None.
        }
    }

    Ok(())
}
