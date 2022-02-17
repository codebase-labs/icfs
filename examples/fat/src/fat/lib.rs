use ic_cdk_macros::{init, query, update};
use std::convert::TryInto;
use std::io::{Read, Write};

// type FileSystem = fatfs::FileSystem<
//     fatfs::StdIoWrapper<fscommon::BufStream<icfs::StableMemory>>,
//     InternetComputerTimeProvider,
//     fatfs::LossyOemCpConverter,
// >;
type FileSystem = fatfs::FileSystem<
    fatfs::StdIoWrapper<icfs::StableMemory>,
    InternetComputerTimeProvider,
    fatfs::LossyOemCpConverter,
>;

thread_local! {
    // static STABLE_MEMORY: std::cell::RefCell<fscommon::BufStream<icfs::StableMemory>>
    //     = std::cell::RefCell::new(fscommon::BufStream::new(icfs::StableMemory::default()));
    static STABLE_MEMORY: std::cell::RefCell<icfs::StableMemory>
        = std::cell::RefCell::new(icfs::StableMemory::default());

    static FS: std::cell::RefCell<FileSystem> = {
        let fs: std::io::Result<FileSystem> = STABLE_MEMORY.with(|stable_memory| {
            let mut stable_memory = *stable_memory.borrow();

            // A Wasm memory page is 2^16 bytes. Canisters have a 4 Gigabyte limit.
            // 4 GB is 2^16 * 2^16 bytes. Apparently we can grow beyond that to 2^17
            // pages.
            stable_memory.grow(2 ^ 17)?;
            // TODO: stable_memory.grow(core::arch::wasm32::memory_size(0))?;

            // TODO:
            // let stable_memory = fscommon::BufStream::new(stable_memory);

            fatfs::format_volume(
                &mut fatfs::StdIoWrapper::from(stable_memory),
                fatfs::FormatVolumeOptions::new(),
            )?;

            let options = fatfs::FsOptions::new()
                .time_provider(InternetComputerTimeProvider::new())
                .update_accessed_date(true);

            let fs = fatfs::FileSystem::new(stable_memory, options)?;

            Ok(fs)
        });

        std::cell::RefCell::new(fs.unwrap())
    };
}

#[init]
fn init() {
    ic_cdk::print("init");

    #[cfg(target_arch = "wasm32")]
    _init().unwrap()
}

#[cfg(target_arch = "wasm32")]
fn _init() -> std::io::Result<()> {
    // FIXME: debugging file not found
    let mut stable_memory = icfs::StableMemory::default();

    let memory_pages = core::arch::wasm32::memory_size(0)
        .try_into()
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))?;

    stable_memory.grow(memory_pages)?;

    // TODO:
    // let stable_memory = fscommon::BufStream::new(stable_memory);

    fatfs::format_volume(
        &mut fatfs::StdIoWrapper::from(stable_memory),
        fatfs::FormatVolumeOptions::new(),
    )?;

    let options = fatfs::FsOptions::new()
        .time_provider(InternetComputerTimeProvider::new())
        .update_accessed_date(true);

    let fs = fatfs::FileSystem::new(stable_memory, options)?;

    // FS.with(|fs| {
    // let fs = fs.borrow();
    let filename = "hello.txt";
    let name = "World!";
    //
    let root_dir = fs.root_dir();
    let mut file = root_dir.create_file(filename)?;
    let contents = format!("Hello, {}!", name).into_bytes();
    file.write_all(&contents)?;
    //
    let entries: std::io::Result<Vec<String>> = root_dir
        .iter()
        .map(|entry| {
            entry
                .map(|e| e.file_name())
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))
        })
        .collect();
    let entries = entries.map(|entries| entries.join("\n"))?;
    ic_cdk::print(format!("entries: {}", entries));
    //
    let mut file = root_dir.open_file(filename)?;
    let mut buf = vec![];
    file.read_to_end(&mut buf)?;
    let contents = String::from_utf8(buf)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))?;
    ic_cdk::print(format!("contents: {}", contents));
    //
    Ok(())
    // })
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
fn read_hello() -> String {
    _read_hello().unwrap()
}

fn _read_hello() -> std::io::Result<String> {
    FS.with(|fs| {
        let fs = fs.borrow();
        let root_dir = fs.root_dir();
        let mut file = root_dir.open_file("hello.txt")?;
        let mut buf = vec![];
        file.read_to_end(&mut buf)?;
        let contents = String::from_utf8(buf)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))?;
        Ok(contents)
    })
}

#[update]
fn write_hello(name: String) {
    _write_hello(name).unwrap();
}

fn _write_hello(name: String) -> std::io::Result<()> {
    FS.with(|fs| {
        let fs = fs.borrow();
        let root_dir = fs.root_dir();
        let mut file = root_dir.create_file("hello.txt")?;
        let contents = format!("Hello, {}!", name).into_bytes();
        file.write_all(&contents)?;
        Ok(())
    })
}

//

#[derive(Debug, Clone, Copy, Default)]
pub struct InternetComputerTimeProvider {
    _dummy: (),
}

impl InternetComputerTimeProvider {
    #[must_use]
    pub fn new() -> Self {
        Self { _dummy: () }
    }
}

impl fatfs::TimeProvider for InternetComputerTimeProvider {
    fn get_current_date(&self) -> fatfs::Date {
        self.get_current_date_time().date
    }

    fn get_current_date_time(&self) -> fatfs::DateTime {
        let ns = time::Duration::nanoseconds(ic_cdk::api::time() as i64);

        let epoch = time::PrimitiveDateTime::new(
            time::Date::from_calendar_date(1970, time::Month::January, 1).unwrap(),
            time::Time::from_hms(0, 0, 0).unwrap(),
        );

        let datetime = epoch.checked_add(ns).unwrap();

        // NOTE: fatfs only supports years in the range [1980, 2107]
        let year: u16 = datetime.year().try_into().unwrap();

        let month = match datetime.month() {
            time::Month::January => 1,
            time::Month::February => 2,
            time::Month::March => 3,
            time::Month::April => 4,
            time::Month::May => 5,
            time::Month::June => 6,
            time::Month::July => 7,
            time::Month::August => 8,
            time::Month::September => 9,
            time::Month::October => 10,
            time::Month::November => 11,
            time::Month::December => 12,
        };

        let day = datetime.day() as u16;

        let hour = datetime.hour() as u16;
        let min = datetime.minute() as u16;
        let sec = datetime.second() as u16;
        let millis = datetime.millisecond() as u16;

        fatfs::DateTime::new(
            fatfs::Date::new(year, month, day),
            fatfs::Time::new(hour, min, sec, millis),
        )
    }
}
