use std::convert::TryInto;

#[derive(Debug, Clone, Copy, Default)]
pub struct TimeProvider {
    _dummy: (),
}

impl TimeProvider {
    #[must_use]
    pub fn new() -> Self {
        Self { _dummy: () }
    }
}

impl fatfs::TimeProvider for TimeProvider {
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
