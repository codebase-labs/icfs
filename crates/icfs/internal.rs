// Copied from: https://github.com/rust-lang/rust/blob/a2af9cf1cf6ccb195eae40cdd793939bc77e7e73/library/std/src/io/mod.rs#L356
// This may be avoidable in future: https://github.com/rust-lang/rfcs/pull/1210
pub (crate) fn default_read_to_end<R: std::io::Read + ?Sized>(r: &mut R, buf: &mut Vec<u8>) -> std::io::Result<usize> {
    let start_len = buf.len();
    let start_cap = buf.capacity();

    let mut initialized = 0; // Extra initialized bytes from previous loop iteration
    loop {
        if buf.len() == buf.capacity() {
            buf.reserve(32); // buf is full, need more space
        }

        let mut read_buf = std::io::ReadBuf::uninit(buf.spare_capacity_mut());

        // SAFETY: These bytes were initialized but not filled in the previous loop
        unsafe {
            read_buf.assume_init(initialized);
        }

        match r.read_buf(&mut read_buf) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }

        if read_buf.filled_len() == 0 {
            return Ok(buf.len() - start_len);
        }

        // store how much was initialized but not filled
        initialized = read_buf.initialized_len() - read_buf.filled_len();
        let new_len = read_buf.filled_len() + buf.len();

        // SAFETY: ReadBuf's invariants mean this much memory is init
        unsafe {
            buf.set_len(new_len);
        }

        if buf.len() == buf.capacity() && buf.capacity() == start_cap {
            // The buffer might be an exact fit. Let's read into a probe buffer
            // and see if it returns `Ok(0)`. If so, we've avoided an
            // unnecessary doubling of the capacity. But if not, append the
            // probe buffer to the primary buffer and let its capacity grow.
            let mut probe = [0u8; 32];

            loop {
                match r.read(&mut probe) {
                    Ok(0) => return Ok(buf.len() - start_len),
                    Ok(n) => {
                        buf.extend_from_slice(&probe[..n]);
                        break;
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(e) => return Err(e),
                }
            }
        }
    }
}
