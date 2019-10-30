mod create;
mod merge;
mod block;
mod errors;

extern crate md5;

use block::Block;
use std::time::Instant;

fn main() {
    let start: u64 = 0;
    let end: u64 = 10_000_000;

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
}
