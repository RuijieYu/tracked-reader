use std::{
    io::{self, ErrorKind, SeekFrom},
    ops::RangeInclusive,
};

#[derive(Debug)]
pub struct Tracker {
    rec: Vec<TrackerEntry>,
    pos: u64,
    /// The size of the stream.  This information is only available after the
    /// first successful [`Seek::seek`] call.
    sz: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum TrackerEntry {
    Entry(RangeInclusive<u64>),
    Error(ErrorKind),
}

/// Perform `u64 - i64`, avoiding data loss due to type casts.
const fn sub_u64_i64(lhs: u64, rhs: i64) -> u64 {
    if rhs.is_negative() {
        lhs + (-rhs) as u64
    } else {
        lhs - rhs as u64
    }
}

impl Tracker {
    pub const fn new() -> Self {
        Self { rec: Vec::new(), pos: 0, sz: None }
    }

    pub fn read(&mut self, read: io::Result<usize>) -> io::Result<usize> {
        let Self { rec: vec, pos, .. } = self;
        vec.push(match &read {
            Err(e) => TrackerEntry::Error(e.kind()),
            Ok(len) => TrackerEntry::Entry({
                let begin = *pos;
                let end = *pos + *len as u64;
                *pos = end;
                begin..=end
            }),
        });
        read
    }

    pub fn seek(
        &mut self,
        from: SeekFrom,
        seek: io::Result<u64>,
    ) -> io::Result<u64> {
        let Self { pos, sz, .. } = self;
        *pos = seek?;

        if let SeekFrom::End(e) = from {
            // sz + e = pos
            *sz = Some(sub_u64_i64(*pos, e))
        }

        Ok(*pos)
    }

    pub fn sz(&self) -> Option<u64> {
        self.sz
    }

    pub fn pos(&self) -> u64 {
        self.pos
    }
}
