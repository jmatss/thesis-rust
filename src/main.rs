mod block;
mod constants;
mod create;
mod digestwithid;
mod error;
mod merge;

use crate::constants::HASH_SIZE;
use crate::create::create_blocks;
use crate::error::ThesisError;
use crate::merge::merge_blocks;
use clap::{App, Arg};
use std::time::Instant;

pub struct Arguments {
    output: String,
    start: u64,
    end: u64,
    buffer_size: u64,
    print_amount: u64,
    amount_of_threads: usize,
}

fn main() -> Result<(), ThesisError> {
    let arguments = parse_arguments();
    let tot_time = Instant::now();

    /*
        STEP 1
        Create blocks. Every block will contain (buffer_size / HASH_SIZE) hashes.
        The blocks will be sorted in DESC and written to disk in files "filename + block_id".
    */
    let mut sub_time = Instant::now();
    let blocks = create_blocks(&arguments)?;
    println!(
        "-- All blocks done! Elapsed time: {} min --",
        sub_time.elapsed().as_secs() / 60
    );

    /*
        STEP 2
        Merges the blocks into one single sorted file "filename".
        Removes hashes from disk as soon as they have been read into memory, no backup.
    */
    sub_time = Instant::now();
    merge_blocks(blocks, &arguments)?;
    println!(
        "-- Everything done! Merging elapsed time: {} min, total elapsed time: {} min --",
        sub_time.elapsed().as_secs() / 60,
        tot_time.elapsed().as_secs() / 60
    );

    Ok(())
}

// Get first char of string.
macro_rules! first {
    ($str:ident) => {
        &$str
            .chars()
            .next()
            .expect("Unable to get first char.")
            .to_string()
    };
}

// Parse arguments from clap's "matches".
macro_rules! parse {
    ($matches:ident, $name:ident) => {
        $matches
            .value_of($name)
            .expect(&format!("Unable to get argument \"{}\".", $name))
            .to_string()
    };

    ($matches:ident, $name:ident, $type_:ty) => {
        $matches
            .value_of($name)
            .unwrap()
            .parse::<$type_>()
            .expect(&format!(
                "Unable to parse \"{}\" from string to {}.",
                $name,
                stringify!($type_),
            ))
    };
}

fn parse_arguments() -> Arguments {
    let o = "output";
    let default_o = "list";
    let s = "start";
    let default_s = (0 as u64).to_string();
    let e = "end";
    let default_e = (0xffff_ffff as u64).to_string();
    let b = "buffer_size";
    let default_b = (((1 << 28) * HASH_SIZE) as u64).to_string(); // 4 GB
    let p = "print_amount";
    let default_p = (200_000_000 as u64).to_string();
    let t = "threads";
    let default_t = (num_cpus::get() as usize).to_string();

    let matches = App::new("thesis-rust")
        .about("Creates a sorted word list with all possible SSID & password combinations.")
        .arg(
            Arg::with_name(o)
                .value_name("PATH")
                .short(first!(o))
                .long(o)
                .help("Name of output file.")
                .takes_value(true)
                .default_value(default_o),
        )
        .arg(
            Arg::with_name(s)
                .value_name("u64")
                .short(first!(s))
                .long(s)
                .help("Start value of serial number.")
                .takes_value(true)
                .default_value(&default_s),
        )
        .arg(
            Arg::with_name(e)
                .value_name("u64")
                .short(first!(e))
                .long(e)
                .help("End value of serial number.")
                .takes_value(true)
                .default_value(&default_e),
        )
        .arg(
            Arg::with_name(b)
                .value_name("u64")
                .short(first!(b))
                .long(b)
                .help("~Buffer size in bytes.")
                .takes_value(true)
                .default_value(&default_b),
        )
        .arg(
            Arg::with_name(p)
                .value_name("u64")
                .short(first!(p))
                .long(p)
                .help("Print status message every \"print_amount\" iteration.")
                .takes_value(true)
                .default_value(&default_p),
        )
        .arg(
            Arg::with_name(t)
                .value_name("usize")
                .short(first!(t))
                .long(t)
                .help("~Max amount of threads.")
                .takes_value(true)
                .default_value(&default_t),
        )
        .get_matches();

    Arguments {
        output: parse!(matches, o),
        start: parse!(matches, s, u64),
        end: parse!(matches, e, u64),
        buffer_size: parse!(matches, b, u64),
        print_amount: parse!(matches, p, u64),
        amount_of_threads: parse!(matches, t, usize),
    }
}
