use crate::constants::{BUF_SIZE, HASH_SIZE, START_CMP};
use crate::error::ThesisError::{CreateError, MergeError};
use crate::error::ThesisResult;
use md5::Digest;
use rayon::prelude::*;
use std::fs::{remove_file, File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::mem;

#[derive(Debug, Clone)]
pub struct Block {
    filename: String,
    start: u64,
    end: u64,
    hashes: Vec<Digest>,
    merge_buffer_size: u64,
}

impl Block {
    // ASCII '0' -> 'f'
    const HEX_LOOKUP: [u8; 16] = [
        48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 97, 98, 99, 100, 101, 102,
    ];

    pub fn new(filename: String, start: u64, end: u64) -> ThesisResult<Block> {
        if end <= start {
            return Err(CreateError(format!(
                "Bad start and end argument: end <= start ({} <= {})",
                end, start
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
            .map(|i| md5::compute(&int_to_serial_number(i)))
            .collect();
        self
    }

    /// Sorts the hashes in self.hashes in DESC order, sorted by their last 6 bytes.
    pub fn sort(&mut self) -> &mut Self {
        self.hashes
            .par_sort_unstable_by(|a, b| b[START_CMP..].cmp(&a[START_CMP..]));
        self
    }

    pub fn write_to_file(&mut self) -> ThesisResult<&mut Self> {
        let file = File::create(&self.filename)?;
        let mut file_writer = BufWriter::with_capacity(BUF_SIZE, file);

        for digest in self.hashes.iter() {
            file_writer.write_all(digest.as_ref())?;
        }
        file_writer.flush()?;

        Ok(self)
    }

    pub fn drop_hashes(&mut self) {
        mem::replace(&mut self.hashes, Vec::with_capacity(0));
    }

    pub fn init_merge(&mut self, mut merge_buffer_size: u64) -> ThesisResult<()> {
        // Floor to multiple of HASH_SIZE.
        merge_buffer_size -= merge_buffer_size % HASH_SIZE as u64;
        if merge_buffer_size == 0 {
            return Err(MergeError(format!(
                "Merge buffer size to small, expected: >={}.",
                HASH_SIZE
            )));
        }

        self.merge_buffer_size = merge_buffer_size;
        self.read()
    }

    pub fn pop(&mut self) -> Option<Digest> {
        if self.hashes.capacity() == 0 || (self.hashes.is_empty() && self.read().is_err()) {
            return None;
        }
        self.hashes.pop()
    }

    /// Reads hashes from hard drive into ram for this block.
    pub fn read(&mut self) -> ThesisResult<()> {
        self.drop_hashes();

        let file = OpenOptions::new()
            .write(true)
            .read(true)
            .open(&self.filename)?;
        let metadata = file.metadata()?;
        let file_length = metadata.len();

        // If the file is empty and there are no hashes left,
        // let self.hashes be the empty vector from the
        // self.drop_hashes() call. The caller will then see
        // that the capacity is zero in the pop function.
        if file_length == 0 {
            mem::drop(file);
            remove_file(&self.filename)?;
            return Ok(());
        }

        // If there are fewer hashes left in the file than there are space in ram,
        // make sure to not read to much (i.e. start reading at 0).
        let seek_start = if self.merge_buffer_size > file_length {
            0
        } else {
            file_length - self.merge_buffer_size
        };

        let mut file_reader = BufReader::with_capacity(BUF_SIZE, &file);
        file_reader.seek(SeekFrom::Start(seek_start))?;

        let amount_of_hashes = (file_length - seek_start) as usize / HASH_SIZE;
        let mut hashes: Vec<Digest> = Vec::with_capacity(amount_of_hashes);

        let mut current_digest: [u8; HASH_SIZE] = [0; HASH_SIZE];
        for _ in 0..amount_of_hashes {
            file_reader.read_exact(&mut current_digest)?;
            hashes.push(Digest(current_digest));
        }

        file.set_len(file_length - (HASH_SIZE * amount_of_hashes) as u64)?;
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
    let mut serial_number: [u8; HASH_SIZE + 1] = [b'0'; HASH_SIZE + 1];
    serial_number[serial_number.len() - 1] = b'\n';

    // Goes through the number "num" shifting it one extra hex char to the right
    // per loop iteration. Masks out the lsB and inserts it into "serial_number".
    for i in (0..=serial_number.len() - 2).rev() {
        if num == 0 {
            break;
        }
        serial_number[i] = Block::HEX_LOOKUP[(0xf & num) as usize];
        num >>= 4;
    }

    serial_number
}
