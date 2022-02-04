mod stable;

use ic_cdk::api::stable::stable64_size;
use stable::{StableReader, StableSeeker, StableWriter};

#[derive(Default)]
pub struct StableMemory {
    reader: StableReader,
    writer: StableWriter,
    seeker: StableSeeker,
}

impl StableMemory {
    pub fn grow(&mut self, added_pages: u64) -> std::io::Result<()> {
        self.writer.grow(added_pages).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Other, "Unable to grow stable memory")
        })
    }

    pub fn size() -> u64 {
        stable64_size()
    }
}

impl std::io::Read for StableMemory {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        <StableReader as std::io::Read>::read(&mut self.reader, buf)
    }
}

impl std::io::Write for StableMemory {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        <StableWriter as std::io::Write>::write(&mut self.writer, buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

impl std::io::Seek for StableMemory {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.seeker.seek(pos)
    }
}
