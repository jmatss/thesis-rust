mod block;
mod cons;
mod create;
mod digestwithid;
mod errors;
mod merge;

extern crate clap;
extern crate md5;
extern crate num_cpus;

use crate::cons::HASH_SIZE;
use crate::create::create_blocks;
use crate::merge::merge_blocks;
use clap::{App, Arg};
use std::time::Instant;

fn main() {
    let (filename, start, end, buffer_size, print_amount, amount_of_threads) = parse_arguments();
    let tot_time = Instant::now();

    /*
        STEP 1
        Create blocks. Every block will contain (buffer_size / HASH_SIZE) hashes.
        The blocks will be sorted in DESC and written to disk in files "filename + block_id".
    */
    let mut time = Instant::now();
    let blocks =
        create_blocks(&filename, start, end, buffer_size).expect("Unable to create blocks");
    println!(
        "-- All blocks done! Elapsed time: {} min --",
        time.elapsed().as_secs() / 60
    );

    /*
        STEP 2
        Merges the blocks into one single sorted file "filename".
        Removes hashes from disk as soon as they have been read into memory, no backup.
    */
    time = Instant::now();
    merge_blocks(
        blocks,
        &filename,
        buffer_size,
        print_amount,
        amount_of_threads,
    )
    .expect("Unable to merge blocks.");

    println!(
        "-- Everything done! Merging elapsed time: {} min, total elapsed time: {} min --",
        time.elapsed().as_secs() / 60,
        tot_time.elapsed().as_secs() / 60
    );
}

/// Returns: (filename, start, end, buffer_size, print_amount, amount_of_threads)
fn parse_arguments() -> (String, u64, u64, u64, u64, usize) {
    const DEFAULT_FILENAME: &str = "list";
    const DEFAULT_START: u64 = 0;
    const DEFAULT_END: u64 = 0xffff_ffff;
    const DEFAULT_BUFFER_SIZE: u64 = (1 << 28) * HASH_SIZE as u64; // 4 GB
    const DEFAULT_PRINT_AMOUNT: u64 = 200_000_000;
    let default_amount_of_threads = num_cpus::get();

    let defstr_start = DEFAULT_START.to_string();
    let defstr_end = DEFAULT_END.to_string();
    let defstr_buffer_size = DEFAULT_BUFFER_SIZE.to_string();
    let defstr_print_amount = DEFAULT_PRINT_AMOUNT.to_string();
    let defstr_amount_of_threads = default_amount_of_threads.to_string();

    let matches = App::new("thesis-rust")
        .about("Creates a sorted word list with all possible SSID & password combinations.")
        .arg(
            Arg::with_name("output")
                .value_name("PATH")
                .short("o")
                .long("output")
                .help("Name of output file.")
                .takes_value(true)
                .default_value(DEFAULT_FILENAME),
        )
        .arg(
            Arg::with_name("start")
                .value_name("u64")
                .short("s")
                .long("start")
                .help("Start value of serial number.")
                .takes_value(true)
                .default_value(&defstr_start),
        )
        .arg(
            Arg::with_name("end")
                .value_name("u64")
                .short("e")
                .long("end")
                .help("End value of serial number.")
                .takes_value(true)
                .default_value(&defstr_end),
        )
        .arg(
            Arg::with_name("buffer_size")
                .value_name("u64")
                .short("b")
                .long("buffer_size")
                .help("~Buffer size in bytes.")
                .takes_value(true)
                .default_value(&defstr_buffer_size),
        )
        .arg(
            Arg::with_name("print_amount")
                .value_name("u64")
                .short("p")
                .long("print_amount")
                .help("Print status message every \"print_amount\" iteration.")
                .takes_value(true)
                .default_value(&defstr_print_amount),
        )
        .arg(
            Arg::with_name("threads")
                .value_name("usize")
                .short("t")
                .long("threads")
                .help("~Max amount of threads.")
                .takes_value(true)
                .default_value(&defstr_amount_of_threads),
        )
        .get_matches();

    let filename = String::from(
        matches
            .value_of("output")
            .expect("Unable to parse \"output\"."),
    );
    let start = matches
        .value_of("start")
        .unwrap()
        .parse::<u64>()
        .expect("Unable to parse \"start\" from string to u64.");
    let end = matches
        .value_of("end")
        .unwrap()
        .parse::<u64>()
        .expect("Unable to parse \"end\" from string to u64.");
    let buffer_size = matches
        .value_of("buffer_size")
        .unwrap()
        .parse::<u64>()
        .expect("Unable to parse \"buffer_size\" from string to u64.");
    let print_amount = matches
        .value_of("print_amount")
        .unwrap()
        .parse::<u64>()
        .expect("Unable to parse \"print_amount\" from string to u64.");
    let amount_of_threads = matches
        .value_of("threads")
        .unwrap()
        .parse::<usize>()
        .expect("Unable to parse \"threads\" from string to usize.");

    (
        filename,
        start,
        end,
        buffer_size,
        print_amount,
        amount_of_threads,
    )
}
