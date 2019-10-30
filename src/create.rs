use std::error::Error;
use std::time::Instant;
use crate::block::Block;
use crate::cons::HASH_SIZE;
use crate::errors::GeneralError;

pub fn create_blocks(
    start: u64,
    end: u64,
    mut buffer_size: u64,
    filename: String,
) -> Result<Vec<Block>, Box<dyn Error>> {
    let amount_of_hashes = end - start + 1;
    let mut blocks: Vec<Block> = Vec::new();

    // Floor to multiple of HASH_SIZE.
    buffer_size -= buffer_size % HASH_SIZE as u64;
    if buffer_size == 0 {
        return Err(Box::new(GeneralError::new(
            format!("Specified buffer size is to small. Needs to be >= {} bytes.", HASH_SIZE)
        )));
    }

    let mut current_start = start;
    let mut current_end;

    for i in 0.. {
        current_end = current_start + (buffer_size / HASH_SIZE as u64);
        if current_end > end {
            current_end = end;
        }

        let mut start_time = Instant::now();
        let mut b = Block::new(format!("{}{}", filename, i), current_start, current_end)?;
        b.generate();
        println!("Block{} generate: {} ms", i, start_time.elapsed().as_millis());

        start_time = Instant::now();
        b.sort();
        println!("Block{} sort: {} ms", i, start_time.elapsed().as_millis());

        start_time = Instant::now();
        b.write_to_file()?;
        println!("Block{} write_to_file: {} ms", i, start_time.elapsed().as_millis());

        b.clear_hashes();

        blocks.push(b);

        if current_end == end {
            break;
        }

        // Setup for next iteration of the for-loop.
        current_start = current_end + 1;
    }

    return Ok(blocks);
}