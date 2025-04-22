use chrono::{DateTime, TimeDelta, TimeZone, Timelike};

use crate::TimeWindow;

/// checks if the window is a factor
/// of 60
pub(crate) fn clamp_window(mut window: TimeWindow) -> Option<TimeWindow> {
    if 60 % window != 0 {
        return None;
    }

    window = window.clamp(1, 60);

    window.into()
}

/// prepares and returns the minute
///
pub(crate) fn prep_time<Tz: TimeZone>(
    datetime: &DateTime<Tz>,
    window: &mut TimeWindow,
) -> Option<u32> {
    *window = clamp_window(*window)?;

    datetime.minute().into()
}

/// Extension trait for simplicity
pub trait ClosestFloor<Tz: TimeZone> {
    /// Get the closest floor of a given time
    /// and window.
    /// window must be a factor of 60
    /// otherwise it returns none.
    /// Seconds will be set to 0.
    /// Window will be clamped to 1-60
    fn closest_floor(&self, window: TimeWindow) -> Option<DateTime<Tz>>
    where
        Self: Sized;
}

impl<Tz: TimeZone> ClosestFloor<Tz> for DateTime<Tz>
where
    <Tz as chrono::TimeZone>::Offset: Copy,
{
    fn closest_floor(&self, mut window: TimeWindow) -> Option<DateTime<Tz>> {
        let start_min = prep_time(self, &mut window)?;
        let closest_floor = (start_min / window) * window;
        self.with_minute(closest_floor)?.with_second(0)?.into()
    }
}

/// Extension trait for simplicity
pub trait ClosestCeil<Tz: TimeZone> {
    /// Get the closest ceil of a given time
    /// and window.
    /// window must be a factor of 60
    /// otherwise it returns none.
    /// Seconds will be set to 0
    fn closest_ceil(&self, window: TimeWindow) -> Option<DateTime<Tz>>
    where
        Self: Sized;
}

impl<Tz: TimeZone + Copy> ClosestCeil<Tz> for DateTime<Tz>
where
    <Tz as chrono::TimeZone>::Offset: Copy,
{
    fn closest_ceil(&self, mut window: TimeWindow) -> Option<DateTime<Tz>> {
        let start_min = prep_time(self, &mut window)?;
        let closest_ceil = start_min.div_ceil(window) * window;

        if closest_ceil >= 60 {
            self.checked_add_signed(TimeDelta::hours(1))?
                .with_minute(0)?
                .with_second(0)
        } else {
            self.with_minute(closest_ceil)?.with_second(0)
        }
    }
}

#[test]
fn test_ext() {
    use chrono::Utc;

    /////////////////////////////////////////////////////////// floor test /////////////////////////////////////////////////////////
    let dt = &Utc.with_ymd_and_hms(2014, 7, 8, 9, 10, 11).unwrap(); // `2014-07-08T09:10:11Z`

    // 60 min window test
    let window = 60;

    // Check 9:00 / 60 min window
    let base = Utc.with_ymd_and_hms(2014, 7, 8, 9, 0, 0).unwrap();

    assert_eq!(dt.closest_floor(window).unwrap(), base);

    // 5 min window test
    let window = 5;

    // Check 9:10 /  5 min window
    let base = Utc.with_ymd_and_hms(2014, 7, 8, 9, 10, 0).unwrap();
    assert_eq!(base, dt.closest_floor(window).unwrap());

    // already the second is 0
    // so it should return this exact same time
    let dt = &Utc.with_ymd_and_hms(2014, 7, 8, 9, 10, 0).unwrap();

    // 5 min window test
    let window = 5;

    // Check 9:10 /  5 min window
    assert_eq!(*dt, dt.closest_floor(window).unwrap());

    ///////////////////////////////////////////////////////// Ceil Test /////////////////////////////////////////////////////////
    let dt = &Utc.with_ymd_and_hms(2014, 7, 8, 9, 12, 11).unwrap(); // `2014-07-08T09:10:11Z`

    // 60 min window test
    let window = 60;

    // Check 10:00 / 60 min window
    let base = Utc.with_ymd_and_hms(2014, 7, 8, 10, 0, 0).unwrap();
    assert_eq!(dt.closest_ceil(window).unwrap(), base);

    // 5 min window test
    let window = 5;

    // Check 9:10 /  5 min window
    let base = Utc.with_ymd_and_hms(2014, 7, 8, 9, 15, 0).unwrap();
    assert_eq!(base, dt.closest_ceil(window).unwrap());

    // already the second is 0
    // so it should return this exact same time
    let dt = &Utc.with_ymd_and_hms(2014, 7, 8, 9, 15, 0).unwrap();
    // 5 min window test
    let window = 5;
    // Check 9:10 /  5 min window
    assert_eq!(*dt, dt.closest_ceil(window).unwrap());
}

#[test]
fn a() {}
