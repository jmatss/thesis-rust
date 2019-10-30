extern crate md5;
extern crate rayon;

use std::error::Error;
use md5::Digest;
use crate::errors::GeneralError;
use rayon::prelude::*;
use std::fs::File;
use std::io::{Write, BufWriter};

#[derive(Debug)]
pub struct Block {
    filename: String,
    start: u64,
    end: u64,
    hashes: Vec<Digest>,
}

impl Block {
    const HASH_SIZE: usize = 16;
    const BUF_WRITER_SIZE: usize = 1 << 16;
    // ASCII '0' -> 'f'
    const HEX_LOOKUP: [u8; 16] = [48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 97, 98, 99, 100, 101, 102];

    pub fn new(filename: String, start: u64, end: u64) -> Result<Block, Box<dyn Error>> {
        if end <= start {
            return Err(Box::new(GeneralError::new(
                format!("Bad start and end argument. end <= start ({} <= {})", end, start)
            )));
        }

        Ok(Block {
            filename,
            start,
            end,
            hashes: Vec::with_capacity(0),
        })
    }

    pub fn generate(&mut self) -> &mut Self {
        self.hashes = (self.start..=self.end)
            .into_par_iter()
            .map(|i| {
                md5::compute(&int_to_serial_number(i))
            })
            .collect();

        self
    }

    /// Sorts the hashes in self.hashes in DESC order, sorted by their last 6 bytes.
    pub fn sort(&mut self) -> &mut Self {
        let start_cmp = 10;
        self.hashes.par_sort_unstable_by(|a, b| {
            b[start_cmp..].cmp(&a[start_cmp..])
        });
        self
    }

    pub fn write_to_file(&mut self) -> Result<&mut Self, Box<dyn Error>> {
        let mut buf = BufWriter::with_capacity(Block::BUF_WRITER_SIZE, File::create(&self.filename)?);
        //let mut buf = BufWriter::new(File::create(&self.filename)?);

        for digest in self.hashes.iter() {
            buf.write(digest.as_ref())?;
        }
        buf.flush()?;

        Ok(self)
    }
}

/// Converts a u64 number to a 17 byte hex string padded with '0' to the left
/// and a '\n' as the last byte.
/// Equivalent to (but faster than):
///
/// ```
/// format!("{:0>16x}\n", num);
/// ```
fn int_to_serial_number(mut num: u64) -> [u8; Block::HASH_SIZE + 1] {
    let mut serial_number: [u8; Block::HASH_SIZE + 1] = ['0' as u8; Block::HASH_SIZE + 1];
    serial_number[serial_number.len() - 1] = '\n' as u8;

    // Goes through the number "num" shifting it one extra hex char to the right
    // per loop iteration. Masks out the lsB and inserts it into "serial_number".
    for i in (0..=serial_number.len() - 2).rev() {
        if num == 0 {
            break;
        }
        serial_number[i] = Block::HEX_LOOKUP[(0xf & num) as usize];
        num >>= 4;
    }

    return serial_number;
}