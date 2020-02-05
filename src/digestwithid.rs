use crate::constants::START_CMP;
use md5::Digest;
use std::cmp::Ordering;

#[derive(Debug)]
pub struct DigestWithID {
    pub id: usize,
    pub digest: Option<Digest>,
}

impl DigestWithID {
    pub fn new(id: usize, digest: Option<Digest>) -> Self {
        DigestWithID { id, digest }
    }
}

/// OBS! Order is in reverse. This is because the default heap is a max heap,
/// so reverse the order of DigestWithID and you get a min heap.
/// Otherwise one would have to wrap every item in a "Reverse".
impl PartialOrd for DigestWithID {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(other.digest?[START_CMP..].cmp(&self.digest?[START_CMP..]))
    }
}

/// OBS! Order is in reverse. This is because the default heap is a max heap,
/// so reverse the order of DigestWithID and you get a min heap.
/// Otherwise one would have to wrap every item in a "Reverse".
///
/// Treats a "None"-option as greater than (so that it sinks).
impl std::cmp::Ord for DigestWithID {
    fn cmp(&self, other: &Self) -> Ordering {
        if let Some(s) = self.digest {
            if let Some(o) = other.digest {
                o[START_CMP..].cmp(&s[START_CMP..])
            } else {
                Ordering::Less
            }
        } else {
            Ordering::Greater
        }
    }
}

impl Eq for DigestWithID {}

impl PartialEq for DigestWithID {
    fn eq(&self, other: &Self) -> bool {
        self.digest == other.digest
    }
}
