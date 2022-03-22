use ic_cdk_macros::{query, update};
use std::io::{Read, Write};

#[cfg(target_arch = "wasm32")]
use std::convert::TryInto;

// type FileSystem = fatfs::FileSystem<
//     fatfs::StdIoWrapper<fscommon::BufStream<icfs::StableMemory>>,
//     icfs_fatfs::TimeProvider,
//     fatfs::LossyOemCpConverter,
// >;
type FileSystem = fatfs::FileSystem<
    fatfs::StdIoWrapper<icfs::StableMemory>,
    icfs_fatfs::TimeProvider,
    fatfs::LossyOemCpConverter,
>;

thread_local! {
    // static STABLE_MEMORY: std::cell::RefCell<fscommon::BufStream<icfs::StableMemory>>
    //     = std::cell::RefCell::new(fscommon::BufStream::new(icfs::StableMemory::default()));
    static STABLE_MEMORY: std::cell::RefCell<icfs::StableMemory>
        = std::cell::RefCell::new(icfs::StableMemory::default());

    static FS: std::cell::RefCell<FileSystem> = {
        let fs: std::io::Result<FileSystem> = STABLE_MEMORY.with(|stable_memory| {
            let stable_memory = *stable_memory.borrow();

            #[cfg(target_arch = "wasm32")]
            let memory_pages = core::arch::wasm32::memory_size(0)
                .try_into()
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))?;

            #[cfg(not(target_arch = "wasm32"))]
            let memory_pages = 19;

            icfs::StableMemory::grow(memory_pages)?;

            // TODO
            // let stable_memory = fscommon::BufStream::new(stable_memory);

            fatfs::format_volume(
                &mut fatfs::StdIoWrapper::from(stable_memory),
                fatfs::FormatVolumeOptions::new(),
            )?;

            let options = fatfs::FsOptions::new()
                .time_provider(icfs_fatfs::TimeProvider::new())
                .update_accessed_date(true);

            let fs = fatfs::FileSystem::new(stable_memory, options)?;

            Ok(fs)
        });

        std::cell::RefCell::new(fs.unwrap())
    };
}

#[query]
fn ls() -> String {
    _ls().unwrap()
}

fn _ls() -> std::io::Result<String> {
    FS.with(|fs| {
        let fs = fs.borrow();
        let root_dir = fs.root_dir();
        let entries: std::io::Result<Vec<String>> = root_dir
            .iter()
            .map(|entry| {
                entry
                    .map(|e| e.file_name())
                    .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))
            })
            .collect();
        entries.map(|entries| entries.join("\n"))
    })
}

#[query]
fn read_file(filename: String) -> String {
    _read_file(filename).unwrap()
}

fn _read_file(filename: String) -> std::io::Result<String> {
    FS.with(|fs| {
        let fs = fs.borrow();
        let root_dir = fs.root_dir();
        let mut file = root_dir.open_file(&filename)?;
        let mut buf = vec![];
        file.read_to_end(&mut buf)?;
        let contents = String::from_utf8(buf)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))?;
        Ok(contents)
    })
}

#[update]
fn write_file(filename: String, contents: String) {
    _write_file(filename, contents).unwrap();
}

fn _write_file(filename: String, contents: String) -> std::io::Result<()> {
    FS.with(|fs| {
        let fs = fs.borrow();
        let root_dir = fs.root_dir();
        let mut file = root_dir.create_file(&filename)?;
        file.truncate()?;
        file.write_all(&contents.into_bytes())?;
        file.flush()?;
        Ok(())
    })
}
