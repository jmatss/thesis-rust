use crate::block::Block;
use crate::constants::{BUF_SIZE, CHAN_BUF_SIZE};
use crate::digestwithid::DigestWithID;
use crate::error::ThesisError::MergeError;
use crate::error::ThesisResult;
use crate::Arguments;
use crossbeam_channel::Sender;
use crossbeam_utils::thread as cb_thread;
use md5::Digest;
use std::collections::BinaryHeap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::thread;

/// Merges the blocks on disk into a single file sorted by their last 6 bytes in ASC.
pub fn merge_blocks(blocks: Vec<Block>, arguments: &Arguments) -> ThesisResult<()> {
    let output = &arguments.output;
    let buffer_size = arguments.buffer_size;
    let print_amount = arguments.print_amount;
    let amount_of_threads = arguments.amount_of_threads;

    if amount_of_threads == 0 {
        return Err(MergeError("Amount of threads set to zero.".to_string()));
    }

    let file = File::create(output)?;
    let mut file_writer = BufWriter::with_capacity(BUF_SIZE, file);
    let (tx_child, rx_child) = crossbeam_channel::bounded(CHAN_BUF_SIZE);
    let merge_handler = thread::spawn(move || -> ThesisResult<()> {
        merge_handler(blocks, amount_of_threads, buffer_size, tx_child)
    });

    // Will receive all hashes in sorted order on the rx_child channel from the
    // spawned "merge_handler" and writes them all into the final output file.
    let mut count: u64 = 0;
    while let Some(digest) = rx_child.recv()? {
        file_writer.write_all(digest.as_ref())?;

        count += 1;
        if count % print_amount == 0 {
            println!("{} hashes merged.", count);
        }
    }
    file_writer.flush()?;

    merge_handler
        .join()
        .map_err(|e| MergeError(format!("Unable to join merge_handler: {:?}", e)))?
}

/// Spawns threads that does all comparisons on the blocks while this merge_handler
/// gathers the results and does comparisons on the results from those threads.
/// Sends the current "ultimate" smallest hash to the "main-thread" over the tx_parent channel.
fn merge_handler(
    mut blocks: Vec<Block>,
    mut amount_of_threads: usize,
    buffer_size: u64,
    tx_parent: Sender<Option<Digest>>,
) -> ThesisResult<()> {
    let buffer_size_per_block = buffer_size / blocks.len() as u64;
    for block in blocks.iter_mut() {
        block.init_merge(buffer_size_per_block)?;
    }

    // Fix edge case if there are fewer block than threads.
    let mut blocks_per_thread = if blocks.len() < amount_of_threads {
        amount_of_threads = blocks.len();
        1
    } else {
        blocks.len() / amount_of_threads
    };

    let mut rx_channels = Vec::with_capacity(amount_of_threads);
    let mut priority_queue = BinaryHeap::with_capacity(amount_of_threads);
    let mut remaining = blocks.as_mut_slice();

    let scope = cb_thread::scope(|s| {
        for i in 0..amount_of_threads {
            if i == amount_of_threads - 1 {
                blocks_per_thread = remaining.len();
            }

            let (current, rest) = remaining.split_at_mut(blocks_per_thread);
            remaining = rest;

            let (tx_child, rx_child) = crossbeam_channel::bounded(CHAN_BUF_SIZE);
            rx_channels.push(rx_child);

            s.spawn(move |_| merge_handler_thread(current, &tx_child));

            let digest_with_id = DigestWithID::new(i, rx_channels[i].recv()?);
            priority_queue.push(digest_with_id);
        }

        loop {
            if priority_queue.is_empty() {
                // Indicate to parent that is done by sending None.
                tx_parent.send(None)?;
                break;
            }

            // Var "next" = Next minimum from same thread as "min".
            if let Some(min) = priority_queue.pop() {
                if let Some(next) = rx_channels[min.id].recv()? {
                    priority_queue.push(DigestWithID::new(min.id, Some(next)));
                }
                tx_parent.send(min.digest)?;
            } else {
                panic!("\"None\" value in merge handlers pq, should never happen");
            }
        }

        Ok(())
    });

    scope.map_err(|e| MergeError(format!("merge_handler unable to merge blocks: {:?}", e)))?
}

/// Does comparisons on a range of blocks. Sends the current "ultimate" smallest
/// hash of the blocks to the parent "merge_handler" over the tx_parent channel.
fn merge_handler_thread(
    sub_blocks: &mut [Block],
    tx_parent: &Sender<Option<Digest>>,
) -> ThesisResult<()> {
    let mut priority_queue = BinaryHeap::with_capacity(sub_blocks.len());

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
            panic!("\"None\" value in merge handler threads pq, should never happen");
        }
    }

    Ok(())
}
