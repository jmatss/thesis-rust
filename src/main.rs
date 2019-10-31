mod block;
mod cons;
mod create;
mod errors;
mod merge;

extern crate md5;

use crate::cons::HASH_SIZE;
use crate::merge::merge_blocks;
use create::create_blocks;
use std::time::Instant;

fn main() {
    let start: u64 = 0;
    let end: u64 = 100;
    //let end: u64 = 10_000_000;
    let buffer_size: u64 = 10_000_000 * HASH_SIZE as u64;
    let amount_of_threads = 4;
    let print_amount = 10_000_000;
    let filename = "list";

    let total_time = Instant::now();
    let mut sub_time = Instant::now();
    let blocks = create_blocks(start, end, buffer_size, filename).unwrap();
    println!(
        "Creating hashes done! Elapsed time: {} s",
        sub_time.elapsed().as_secs()
    );

    sub_time = Instant::now();
    merge_blocks(
        blocks,
        amount_of_threads,
        buffer_size,
        filename,
        print_amount,
    )
    .expect("Unable to merge blocks.");
    println!(
        "Everything done! Merging elapsed time: {}, Total elapsed time: {} ms",
        sub_time.elapsed().as_secs(),
        total_time.elapsed().as_secs()
    );

    /*
    let mut start_time = Instant::now();
    let mut b = Block::new(String::from("test"), start, end).expect("Unable to create block.");
    b.generate();
    println!("generate: {}", start_time.elapsed().as_millis());
    start_time = Instant::now();
    b.sort();
    println!("sort: {}", start_time.elapsed().as_millis());
    start_time = Instant::now();
    b.write_to_file()
        .expect("Unable to write to file");
    //println!("{:?}\n\n", b);
    println!("write_to_file: {}", start_time.elapsed().as_millis());

    b.clear_hashes();
    */
}
