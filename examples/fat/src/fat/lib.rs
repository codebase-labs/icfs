use ic_cdk_macros::update;
use std::io::Write;

fn _write_hello(name: String) -> std::io::Result<()> {
    let new_file = icfs::StableMemory::default();
    let buf_stream = fscommon::BufStream::new(new_file);
    let options = fatfs::FsOptions::new().update_accessed_date(true);
    let fs = fatfs::FileSystem::new(buf_stream, options)?;
    let root_dir = fs.root_dir();
    let mut file = root_dir.create_file("hello.txt")?;
    let contents = format!("Hello, {}!", name).into_bytes();
    file.write_all(&contents)
}

#[update]
fn write_hello(name: String) {
    _write_hello(name).unwrap();
}
