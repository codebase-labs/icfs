use ic_cdk_macros::{init, update};
use std::io::Write;

thread_local! {
    static FS: std::cell::RefCell<fscommon::BufStream<icfs::StableMemory>>
        = std::cell::RefCell::new(fscommon::BufStream::new(icfs::StableMemory::default()));
}

#[init]
fn init() {
    _init().unwrap();
}

fn _init() -> Result<(), std::io::Error> {
    ic_cdk::api::stable::stable64_grow(1024 * 128).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::Other, "Unable to grow stable memory")
    })?;

    // TODO: FS.with(|fs| )
    let fs = icfs::StableMemory::default();
    let fs = fscommon::BufStream::new(fs);

    fatfs::format_volume(
        &mut fatfs::StdIoWrapper::from(fs),
        fatfs::FormatVolumeOptions::new().fats(1),
    )
    .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))
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
