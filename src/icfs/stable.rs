// * Based on https://github.com/dfinity/cdk-rs/blob/6c0aa52bc070c4d31a2ca571799f8a11ceb2d6de/src/ic-cdk/src/api/stable.rs
// * Supports 64-bit addressed memory
// * Aims to prevent out-of-bounds reads
use ic_cdk::api::stable::{stable64_grow, stable64_read, stable64_size, stable64_write, StableMemoryError};
use std::io;

/// A writer to the stable memory.
///
/// Will attempt to grow the memory as it writes,
/// and keep offsets and total capacity.
pub struct StableWriter {
    /// The offset of the next write.
    offset: usize,

    /// The capacity, in pages.
    capacity: u64,
}

impl Default for StableWriter {
    fn default() -> Self {
        let capacity = stable64_size();

        Self {
            offset: 0,
            capacity,
        }
    }
}

impl StableWriter {
    /// Attempts to grow the memory by adding new pages.
    pub fn grow(&mut self, added_pages: u64) -> Result<(), StableMemoryError> {
        let old_page_count = stable64_grow(added_pages)?;
        self.capacity = old_page_count + added_pages;
        Ok(())
    }

    /// Writes a byte slice to the buffer.
    ///
    /// The only condition where this will
    /// error out is if it cannot grow the memory.
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, StableMemoryError> {
        if self.offset + buf.len() > ((self.capacity as usize) << 16) {
            self.grow((buf.len() >> 16) as u64 + 1)?;
        }

        stable64_write(self.offset as u64, buf);
        self.offset += buf.len();
        Ok(buf.len())
    }
}

impl io::Write for StableWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        self.write(buf)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Out Of Memory"))
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        // Noop.
        Ok(())
    }
}


/// A reader to the stable memory.
///
/// Keeps an offset and reads off stable memory consecutively.
pub struct StableReader {
    /// The offset of the next read.
    offset: usize,
}

impl Default for StableReader {
    fn default() -> Self {
        Self { offset: 0 }
    }
}

impl StableReader {
    /// Reads data from the stable memory location specified by an offset.
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, StableMemoryError> {
        let capacity = stable64_size();
        // Aim to address https://github.com/dfinity/cdk-rs/issues/78
        if capacity < buf.len() as u64 {
            Err(StableMemoryError())
        } else {
            stable64_read(self.offset as u64, buf);
            self.offset += buf.len();
            Ok(buf.len())
        }
    }
}

impl io::Read for StableReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        self.read(buf)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Attempt to read beyond available memory"))
    }
}


/// A seeker to the stable memory.
///
/// Keeps an offset.
pub struct StableSeeker {
    offset: usize,
}

impl Default for StableSeeker {
    fn default() -> Self {
        Self { offset: 0 }
    }
}

impl StableSeeker {
    fn seek(&mut self, pos: io::SeekFrom) -> Result<u64, StableMemoryError> {
        match pos {
            io::SeekFrom::Start(start) => {
                self.offset = start as usize;
                Ok(self.offset as u64)
            }
            io::SeekFrom::End(end) => {
                let capacity = stable64_size();
                if capacity as i64 + end >= 0 {
                    self.offset = ((capacity as usize) << 16) + (end as usize);
                    Ok(self.offset as u64)
                } else {
                    Err(StableMemoryError {})
                }
            }
            io::SeekFrom::Current(current) => {
                if self.offset as i64 + current >= 0 {
                    self.offset += current as usize;
                    Ok(self.offset as u64)
                } else {
                    Err(StableMemoryError {})
                }
            }
        }
    }
}

impl io::Seek for StableSeeker {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.seek(pos)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Attempt to seek before byte 0"))
    }
}
