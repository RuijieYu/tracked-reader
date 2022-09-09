use std::io::{self, Read, Seek};

use crate::tracker::Tracker;

#[derive(Debug)]
pub struct Reader<R: Read + Seek>(R, Tracker);

impl<R: Read + Seek> Reader<R> {
    /// Create a reader.  This will rewind the passed-in object.
    pub fn new(mut reader: R) -> io::Result<Self> {
        reader.rewind()?;
        Ok(Self(reader, Tracker::new()))
    }

    /// Expose the tracker.
    pub fn tracker(&self) -> &Tracker {
        &self.1
    }
}

impl<R: Read + Seek> Seek for Reader<R> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        let Self(r, track) = self;
        track.seek(pos, r.seek(pos))
    }
}

impl<R: Read + Seek> Read for Reader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let Self(r, track) = self;
        track.read(r.read(buf))
    }
}
