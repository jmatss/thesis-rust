extern crate md5;

use std::error::Error;
use std::fmt;
use md5::Digest;
use std::thread;
use std::thread::JoinHandle;
use crate::errors::GeneralError;

pub struct Block {
    id: u64,
    start: u64,
    end: u64,
    hashes: Vec<Digest>,
}

impl Block {
    fn new(id: u64, start: u64, end: u64) -> Result<Block, Box<dyn Error>> {
        if end <= start {
            return Err(Box::new(GeneralError::new(
                format!("Bad start and end argument. end <= start ({} <= {})", end, start)
            )));
        }

        let n = (end - start + 1) as usize;
        let mut hashes = Vec::with_capacity(n);
        // Set correct length with initialized values so that the vector can be
        // divided up between the threads without the need to init with default values.
        unsafe { hashes.set_len(n) }
        Ok(Block {
            id,
            start,
            end,
            hashes,
        })
    }

    fn generate(&mut self, mut amount_of_threads: usize) -> Result<(), Box<dyn Error>> {
        let amount_of_hashes = self.hashes.capacity();
        let global_start = self.start;

        // If there are fewer hashes to generate than threads,
        // just spawn one thread and let it create all the hashes.
        if amount_of_hashes < amount_of_threads {
            amount_of_threads = 1;
        }

        let mut threads: Vec<_> = Vec::with_capacity(amount_of_threads);
        let mut hash_range = amount_of_hashes / amount_of_threads;

        let mut rest = self.hashes.as_mut_slice();
        let mut split_index = hash_range;

        for i in 0..amount_of_threads {
            // Last iteration, the last thread will take the rest of the hashes.
            if i == amount_of_threads - 1 {
                split_index = rest.len();
            }

            let (current, rest_tmp) = rest.split_at_mut(split_index);
            rest = rest_tmp;

            // TODO: Make the closure take the "current" directly instead of making a copy "c".
            let mut c = current.to_owned();

            threads.push(thread::spawn(move || {
                let start = global_start + i as u64 * hash_range as u64;
                let end = start + c.len() as u64 - 1;

                // TODO: See todo above.
                generate_sub_block(global_start, start, end, c.as_mut_slice())
                //generate_sub_block(self.start, start, end, current)
            }));
        }

        for thread in threads {
            // TODO: Make so that the error casting can be done in the GeneralError class.
            // thread.join()??;
            thread.join().unwrap_or_else(|_| Err(Box::new(GeneralError::new(String::from("abc")))))?;
        }

        Ok(())
    }
}

fn generate_sub_block(global_start: u64, start: u64, end: u64, hashes: &mut [Digest]) -> Result<(), Box<GeneralError>> {
    if end - start + 1 != hashes.len() as u64 {
        return Err(Box::new(GeneralError::new(
            format!("Given start and end in \"generate_sub_block\" doesn't \
             correspond to the size of the hashes slice. end - start + 1 = {}, \
             hashes.len() = {}", end - start + 1, hashes.len() )
        )));
    }

    for i in start..=end {
        hashes[(i - global_start) as usize] = md5::compute(format!("{:0>16x}", i));
    }

    Ok(())
}