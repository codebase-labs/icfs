use ic_cdk_macros::{query, update};

thread_local! {
    static STABLE_MEMORY: std::cell::RefCell<icfs::StableMemory>
        = std::cell::RefCell::new(icfs::StableMemory::default());

    static VOL: std::cell::RefCell<ext4::SuperBlock<icfs::StableMemory>> = {
        let vol: Result<ext4::SuperBlock<icfs::StableMemory>> = STABLE_MEMORY.with(|stable_memory| {
            let stable_memory = *stable_memory.borrow();

            #[cfg(target_arch = "wasm32")]
            let memory_pages = core::arch::wasm32::memory_size(0)
                .try_into()
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))?;

            #[cfg(not(target_arch = "wasm32"))]
            let memory_pages = 19;

            icfs::StableMemory::grow(memory_pages)?;

            let mut options = ext4::Options::default();
            options.checksums = ext4::Checksums::Enabled;

            let vol = ext4::SuperBlock::new_with_options(stable_memory, &options).expect("ext4 volume");

            Ok(vol)
        });

        std::cell::RefCell::new(vol.unwrap())
    }
}

#[query]
fn ls() -> Vec<String> {
    _ls().unwrap()
}

fn _ls() -> std::io::Result<Vec<String>> {
  Ok(vec!("FIXME".to_string()))
}

#[query]
fn read_file(filename: String) -> String {
    _read_file(filename).unwrap()
}

fn _read_file(filename: String) -> std::io::Result<String> {
    Ok("FIXME".to_string())
}

#[update]
fn write_file(filename: String, contents: String) {
    _write_file(filename, contents).unwrap();
}

fn _write_file(filename: String, contents: String) -> std::io::Result<()> {
    Ok(())
}
