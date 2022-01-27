use ic_cdk_macros::{init, update};
use std::io::Write;
use std::io::Seek;

thread_local! {
    static FS: std::cell::RefCell<fscommon::BufStream<icfs::StableMemory>>
        = std::cell::RefCell::new(fscommon::BufStream::new(icfs::StableMemory::default()));
}

#[init]
fn init() {
    _init().unwrap();
}

fn _init() -> Result<(), std::io::Error> {
    // TODO: FS.with(|fs| )
    let mut stable_memory = icfs::StableMemory::default();

    // A Wasm memory page is 2^16 bytes. Canisters have a 4 Gigabyte limit. 4 GB
    // is 2^16 * 2^16 bytes. Apparently we can grow beyond that to 2^17 pages.
    stable_memory.grow(2^17)?;

    let stable_memory = fscommon::BufStream::new(stable_memory);

    fatfs::format_volume(
        &mut fatfs::StdIoWrapper::from(stable_memory),
        fatfs::FormatVolumeOptions::new(),
    )
    .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))?;

    // TEMP
    let options = fatfs::FsOptions::new().update_accessed_date(true);
    let fs = fatfs::FileSystem::new(stable_memory, options)?;

    let name = "World";

    let root_dir = fs.root_dir();
    let mut file = root_dir.create_file("hello.txt")?;
    let contents = format!("Hello, {}!", name).into_bytes();
    file.write_all(&contents)
}

#[update]
fn write_hello(name: String) {
    _write_hello(name).unwrap();
}

fn _write_hello(name: String) -> std::io::Result<()> {
    // let root_dir = fs.root_dir();
    // let mut file = root_dir.create_file("hello.txt")?;
    // let contents = format!("Hello, {}!", name).into_bytes();
    // file.write_all(&contents)
    Ok(())
}
