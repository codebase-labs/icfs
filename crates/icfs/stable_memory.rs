// Based on https://github.com/dfinity/cdk-rs/blob/a253119adb08929b6304d007ee0a6a37960656ed/src/ic-cdk/src/api/stable.rs
// * Supports 64-bit addressed memory
use ic_cdk::api::stable::{
    stable64_grow, stable64_read, stable64_size, stable64_write, StableMemoryError,
};
use std::io;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct StableMemory {
    offset: usize,
}

fn get_offset(stable_memory: &StableMemory) -> usize {
    stable_memory.offset
}

fn set_offset(stable_memory: &mut StableMemory, offset: usize) {
    stable_memory.offset = offset
}

/// Returns a copy of the stable memory.
///
/// This will map the whole memory (even if not all of it has been written to).
pub fn bytes() -> Vec<u8> {
    let capacity = capacity();
    let mut vec = Vec::with_capacity(capacity);
    unsafe {
        vec.set_len(capacity);
    }
    stable64_read(0, vec.as_mut_slice());
    vec
}

/// Gets capacity of the stable memory in bytes.
pub fn capacity() -> usize {
    (size() as usize) << 16
}

/// Attempts to grow the memory by adding new pages.
pub fn grow(added_pages: u64) -> Result<u64, StableMemoryError> {
    stable64_grow(added_pages)
}

/// Gets current size of the stable memory in WebAssembly pages.
pub fn size() -> u64 {
    stable64_size()
}

/// Reads data from the stable memory location specified by an offset.
pub fn read(stable_memory: &mut StableMemory, buf: &mut [u8]) -> Result<usize, StableMemoryError> {
    let offset = get_offset(stable_memory);
    let capacity = capacity();
    let read_buf = if buf.len() + offset > capacity {
        if offset < capacity {
            &mut buf[..capacity - offset]
        } else {
            return Err(StableMemoryError::OutOfBounds);
        }
    } else {
        buf
    };
    stable64_read(offset as u64, read_buf);
    set_offset(stable_memory, offset + read_buf.len());
    Ok(read_buf.len())
}

fn seek(stable_memory: &mut StableMemory, pos: io::SeekFrom) -> Result<u64, StableMemoryError> {
    match pos {
        io::SeekFrom::Start(start) => {
            set_offset(stable_memory, start as usize);
            Ok(get_offset(stable_memory) as u64)
        }
        io::SeekFrom::End(end) => {
            let new_offset = capacity() as i64 + end;
            if new_offset >= 0 {
                set_offset(stable_memory, new_offset as usize);
                Ok(get_offset(stable_memory) as u64)
            } else {
                Err(StableMemoryError::OutOfBounds)
            }
        }
        io::SeekFrom::Current(current) => {
            let new_offset = get_offset(stable_memory) as i64 + current;
            if new_offset >= 0 {
                set_offset(stable_memory, new_offset as usize);
                Ok(get_offset(stable_memory) as u64)
            } else {
                Err(StableMemoryError::OutOfBounds)
            }
        }
    }
}

/// Writes a byte slice to the buffer.
///
/// The only condition where this will
/// error out is if it cannot grow the memory.
pub fn write(stable_memory: &mut StableMemory, buf: &[u8]) -> Result<usize, StableMemoryError> {
    let offset = get_offset(stable_memory);
    let new_offset = offset + buf.len();
    if new_offset > capacity() {
        grow((buf.len() >> 16) as u64 + 1)?;
    }
    stable64_write(offset as u64, buf);
    set_offset(stable_memory, new_offset);
    Ok(buf.len())
}

impl StableMemory {
    /// Returns a copy of the stable memory.
    ///
    /// This will map the whole memory (even if not all of it has been written to).
    pub fn bytes() -> Vec<u8> {
        bytes()
    }

    /// Gets capacity of the stable memory in bytes.
    pub fn capacity() -> usize {
        capacity()
    }

    /// Attempts to grow the memory by adding new pages.
    pub fn grow(added_pages: u64) -> std::io::Result<u64> {
        grow(added_pages).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Other, "Unable to grow stable memory")
        })
    }

    /// Gets current size of the stable memory in WebAssembly pages.
    pub fn size() -> u64 {
        size()
    }
}

impl Default for StableMemory {
    fn default() -> Self {
        Self { offset: 0 }
    }
}

impl std::io::Read for StableMemory {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // self.read(buf).or(Ok(0)) // Read defines EOF to be success
        read(self, buf).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}

impl std::io::Write for StableMemory {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        write(self, buf).map_err(|e| io::Error::new(io::ErrorKind::OutOfMemory, e))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // No-op.
        Ok(())
    }
}

impl std::io::Seek for StableMemory {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        seek(self, pos)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Attempt to seek before byte 0"))
    }
}
