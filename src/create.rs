use crate::block::Block;
use crate::r#const::HASH_SIZE;
use crate::error::ThesisResult;
use std::time::Instant;
use crate::Arguments;
use crate::error::ThesisError::CreateError;

/// Creates all blocks and writes them to individually sorted files in DESC.
/// Returns the "meta data" of the blocks i.e. the "block" structs without the hashes.
pub fn create_blocks(arguments: &Arguments) -> ThesisResult<Vec<Block>> {
    let mut blocks: Vec<Block> = Vec::new();

    let output = &arguments.output;
    let start = arguments.start;
    let end = arguments.end;
    let mut buffer_size = arguments.buffer_size;

    // Floor to multiple of HASH_SIZE.
    buffer_size -= buffer_size % HASH_SIZE as u64;
    if buffer_size == 0 {
        return Err(CreateError(
            format!(
                "Specified buffer size is to small. Needs to be >= {} bytes.",
                HASH_SIZE
            )
        ));
    }

    let mut current_start = start;
    let mut current_end;

    for i in 0.. {
        current_end = current_start + (buffer_size / HASH_SIZE as u64);
        if current_end > end {
            current_end = end;
        }

        let time = Instant::now();

        let block_filename = format!("{}{}", output, i);
        let mut block = Block::new(block_filename, current_start, current_end)?;
        block.generate().sort().write_to_file()?.drop_hashes();

        println!(
            "Block{} created and written to file: {} sec",
            i,
            time.elapsed().as_secs()
        );

        blocks.push(block);

        if current_end == end {
            break;
        }

        // Setup for next iteration of the for-loop.
        current_start = current_end + 1;
    }

    Ok(blocks)
}
