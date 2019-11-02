mod block;
mod cons;
mod create;
mod errors;
mod merge;

extern crate md5;
extern crate num_cpus;

use crate::cons::HASH_SIZE;
use crate::merge::merge_blocks;
use create::create_blocks;
use std::time::Instant;

fn main() {
    let start: u64 = 0;
    let end: u64 = 0xffff_ffff;
    let buffer_size: u64 = (1 << 28) * HASH_SIZE as u64; // 4 GB
    let amount_of_threads = num_cpus::get();
    let print_amount = 10_000_000;
    let filename = "list";

    let tot_time = Instant::now();

    /*
        STEP 1
        Create blocks. Every block will contain (buffer_size / HASH_SIZE) hashes.
        The blocks will be sorted in DESC and written to disk in files "filename + block_id".
    */
    let mut time = Instant::now();
    let blocks = create_blocks(start, end, buffer_size, filename).expect("Unable to create blocks");
    println!(
        "Creating hashes done! Elapsed time: {} s",
        time.elapsed().as_secs()
    );

    /*
        STEP 2
        Merges the blocks into one single sorted file "filename".
        Removes hashes from disk as soon as they have been read into memory, no backup.
    */
    time = Instant::now();
    merge_blocks(
        blocks,
        amount_of_threads,
        buffer_size,
        filename,
        print_amount,
    )
    .expect("Unable to merge blocks.");
    println!(
        "Everything done! Merging elapsed time: {} ms, Total elapsed time: {} min",
        time.elapsed().as_secs(),
        tot_time.elapsed().as_secs() / 60
    );
}
