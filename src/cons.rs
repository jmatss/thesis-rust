// Size in bytes(u8) of a md5 hash.
pub const HASH_SIZE: usize = 16;

// SSIDs in hashes starts at byte 10.
pub const START_CMP: usize = 10;

// Used by BufWriters and BufReaders (arbitrary value).
pub const BUF_SIZE: usize = 1 << 16;

// Used by channels (arbitrary value).
pub const CHAN_BUF_SIZE: usize = 1 << 16;
