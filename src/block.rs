extern crate md5;
extern crate rayon;

use md5::Digest;
use rayon::prelude::*;
use std::error::Error;
use std::fs::File;
use std::io::{Write, BufWriter, Seek, SeekFrom, BufReader, Read};
use crate::cons::{HASH_SIZE, START_CMP};
use crate::errors::GeneralError;

#[derive(Debug, Clone)]
pub struct Block {
    filename: String,
    start: u64,
    end: u64,
    hashes: Vec<Digest>,
    merge_buffer_size: u64,
}

impl Block {
    // Used by BufWriters and BufReaders.
    const BUF_SIZE: usize = 1 << 16;
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
            merge_buffer_size: 0,
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
        self.hashes.par_sort_unstable_by(|a, b| {
            b[START_CMP..].cmp(&a[START_CMP..])
        });
        self
    }

    pub fn write_to_file(&mut self) -> Result<&mut Self, Box<dyn Error>> {
        let mut buf = BufWriter::with_capacity(Block::BUF_SIZE, File::create(&self.filename)?);
        //let mut buf = BufWriter::new(File::create(&self.filename)?);

        for digest in self.hashes.iter() {
            buf.write(digest.as_ref())?;
        }
        buf.flush()?;

        Ok(self)
    }

    /// Drop the hashes from memory so that only the "meta-data" of the hashes are loft in the struct.
    pub fn clear_hashes(&mut self) {
        std::mem::drop(std::mem::replace(&mut self.hashes, Vec::with_capacity(0)));
    }

    pub fn init_merge(&mut self, mut merge_buffer_size: u64) {
        merge_buffer_size -= merge_buffer_size % HASH_SIZE as u64;
        /*
        if merge_buffer_size == 0 {
            return Err(Box::new(GeneralError::new(String::from("Merge buffer size to small"))));
        }
        */
        self.merge_buffer_size = merge_buffer_size;
        //self.read()?;

        //Ok(())
    }

    pub fn pop(&mut self) -> Option<Digest> {
        if self.hashes.capacity() == 0 {
            return None;
        } else if self.hashes.is_empty() {
            self.read().ok();
        }
        self.hashes.pop()
    }

    fn read(&mut self) -> Result<(), Box<dyn Error>> {
        self.clear_hashes();

        let file = File::open(&self.filename)?;
        let metadata = file.metadata()?;
        let file_length = metadata.len();

        // If the file is empty and there are no hashes left,
        // return an empty vector that the caller can use to see
        // that this block is out of hashes.
        if file_length == 0 {
            self.hashes = Vec::with_capacity(0);
            return Ok(());
        }

        // If there are fewer hashes left in the file than there are space in ram,
        // make sure to not read to much (i.e. start reading at 0).
        let seek_start = if self.merge_buffer_size > file_length {
            0
        } else {
            file_length - self.merge_buffer_size
        };

        let mut buf = BufReader::with_capacity(Block::BUF_SIZE, file);
        buf.seek(SeekFrom::Start(seek_start))?;

        let amount_of_hashes = (file_length - seek_start) as usize / HASH_SIZE;
        let mut hashes: Vec<Digest> = Vec::with_capacity(amount_of_hashes);

        let mut current_digest: [u8; HASH_SIZE] = [0; HASH_SIZE];
        for i in 0..amount_of_hashes {
            buf.read(&mut current_digest)?;
            hashes.push(Digest(current_digest));
        }

        self.hashes = hashes;

        Ok(())
    }
}

/// Converts a u64 number to a 17 byte hex string padded with '0' to the left
/// and a '\n' as the last byte.
/// Equivalent to (but faster than):
///
/// ```
/// format!("{:0>16x}\n", num);
/// ```
fn int_to_serial_number(mut num: u64) -> [u8; HASH_SIZE + 1] {
    let mut serial_number: [u8; HASH_SIZE + 1] = ['0' as u8; HASH_SIZE + 1];
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