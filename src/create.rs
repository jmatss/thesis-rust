use crate::block::Block;
use crate::cons::HASH_SIZE;
use crate::errors::GeneralError;
use std::error::Error;
use std::time::Instant;

pub fn create_blocks(
    start: u64,
    end: u64,
    mut buffer_size: u64,
    filename: &str,
) -> Result<Vec<Block>, Box<dyn Error>> {
    let mut blocks: Vec<Block> = Vec::new();

    // Floor to multiple of HASH_SIZE.
    buffer_size -= buffer_size % HASH_SIZE as u64;
    if buffer_size == 0 {
        return Err(Box::new(GeneralError::new(format!(
            "Specified buffer size is to small. Needs to be >= {} bytes.",
            HASH_SIZE
        ))));
    }

    let mut current_start = start;
    let mut current_end;

    for i in 0.. {
        current_end = current_start + (buffer_size / HASH_SIZE as u64);
        if current_end > end {
            current_end = end;
        }
        /*
                let start_time = Instant::now();
                let mut block = Block::new(format!("{}{}", filename, i), current_start, current_end)?;
                block.generate().sort().write_to_file()?.drop_hashes();
                println!(
                    "Block{} created and written to file: {} sec",
                    i,
                    start_time.elapsed().as_secs()
                );
        */

        let mut start_time = Instant::now();
        let mut block = Block::new(format!("{}{}", filename, i), current_start, current_end)?;
        block.generate();
        println!(
            "Block{} generate: {} ms",
            i,
            start_time.elapsed().as_millis()
        );

        start_time = Instant::now();
        block.sort();
        println!("Block{} sort: {} ms", i, start_time.elapsed().as_millis());

        start_time = Instant::now();
        block.write_to_file()?;
        println!(
            "Block{} write_to_file: {} ms",
            i,
            start_time.elapsed().as_millis()
        );

        block.drop_hashes();

        blocks.push(block);

        if current_end == end {
            break;
        }

        // Setup for next iteration of the for-loop.
        current_start = current_end + 1;
    }

    Ok(blocks)
}
