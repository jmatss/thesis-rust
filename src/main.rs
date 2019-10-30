mod block;
mod cons;
mod create;
mod errors;
mod merge;

extern crate md5;

use block::Block;
use create::create_blocks;
use std::time::Instant;
use crate::cons::HASH_SIZE;

fn main() {
    let start: u64 = 0;
    let end: u64 = 10_000_000;
    let buffer_size: u64 = 10_000_000 * HASH_SIZE as u64;

    let filename = String::from("list");

    let start_time = Instant::now();
    let blocks = create_blocks(start, end, buffer_size, filename).unwrap();
    println!("DONE! Total elapsed: {} ms", start_time.elapsed().as_millis());

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
