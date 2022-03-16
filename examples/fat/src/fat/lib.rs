use ic_cdk_macros::{query, update};
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

// #[cfg(target_arch = "wasm32")]
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
                .time_provider(InternetComputerTimeProvider::new())
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

// TODO: move to own module

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
