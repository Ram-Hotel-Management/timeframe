mod error;
mod ext;
use chrono::{DateTime, TimeZone, Utc};
pub use error::*;
pub use ext::*;
pub type TimeWindow = u32;

pub type TimeframeUtc = Timeframe<Utc>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timeframe<Tz: TimeZone> {
    pub start: DateTime<Tz>,
    pub end: DateTime<Tz>,
}

impl<Tz: TimeZone> Copy for Timeframe<Tz> where <Tz as chrono::TimeZone>::Offset: std::marker::Copy {}

impl<Tz: TimeZone + Copy> Timeframe<Tz>
where
    <Tz as chrono::TimeZone>::Offset: Copy,
{
    /// convert into an even timeframe
    pub fn into_even(self, window: TimeWindow) -> EvenTimeframe<Tz> {
        EvenTimeframe::new(self.start, window)
    }
}

fn closest_factor_of_60(window: TimeWindow) -> TimeWindow {
    let window = window.clamp(0, 60);
    const FACTORS_60: [TimeWindow; 12] = [1, 2, 3, 4, 5, 6, 10, 12, 15, 20, 30, 60];
    *FACTORS_60
        .iter()
        .min_by_key(|n| window.abs_diff(**n))
        .unwrap()
}

pub type EvenTimeframeUtc = EvenTimeframe<Utc>;

/// Even timeframe ensures the provided timeframe is
/// always evenly split where the seconds & milliseconds is always 00 and floored to closest window multiple.
/// For instance
/// - window size: 30 & time: 12:38:45 -> 12:30:00
/// - window size: 15 & time: 12:14:12 -> 12:00:00
/// - window size: 5 & time: 00:05:01 -> 00:05:00
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EvenTimeframe<Tz: TimeZone> {
    pub frame: Timeframe<Tz>,
    window: TimeWindow,
}

/// Easy Access functions
impl<Tz: TimeZone> EvenTimeframe<Tz> {
    pub fn start(&self) -> &DateTime<Tz> {
        &self.frame.start
    }

    pub fn end(&self) -> &DateTime<Tz> {
        &self.frame.end
    }

    /// Get the timeframe window
    pub fn get_window(&self) -> TimeWindow {
        self.window
    }

    /// convert the Type to Utc
    /// short hand function
    pub fn to_utc(&self) -> EvenTimeframeUtc {
        EvenTimeframe::<chrono::Utc> {
            frame: Timeframe {
                start: self.start().to_utc(),
                end: self.end().to_utc(),
            },
            window: self.window,
        }
    }
}

impl<Tz: TimeZone + Copy> EvenTimeframe<Tz>
where
    <Tz as chrono::TimeZone>::Offset: Copy,
{
    /// Chops up the given timeframe after flooring the start time
    /// and ceiling the end time based on the provided window
    pub fn split(mut frame: Timeframe<Tz>, mut window: TimeWindow) -> Result<Vec<Self>, TimeErr> {
        window = clamp_window(window).ok_or_else(|| {
            TimeErr::Other("An error occurred while clamping the window size. Ensure the window size is factor of 60.".into())
        })?;
        frame.start = frame.start.closest_floor(window).ok_or(TimeErr::Floor)?;
        frame.end = frame.end.closest_ceil(window).ok_or(TimeErr::Ceil)?;
        let delta = frame.end - frame.start;

        if delta.num_weeks() >= 15 {
            return Err(TimeErr::FrameTooLarge);
        }

        if delta.num_minutes() <= 0 {
            return Err(TimeErr::Other(
                "Difference between start and end must be greater than 1".into(),
            ));
        }

        let mut curr = frame.start;
        let mut time_frames = Vec::new();
        while curr < frame.end {
            let end = curr + chrono::Duration::minutes(window as i64);
            let fr = EvenTimeframe {
                frame: Timeframe { start: curr, end },
                window,
            };

            curr = end;

            time_frames.push(fr);
        }

        Ok(time_frames)
    }

    /// time frame to adjust this will floor to closest window multiple.
    /// Expects window to be a factor 60
    pub fn new(start: DateTime<Tz>, window: TimeWindow) -> Self {
        let window = closest_factor_of_60(window);

        let frame = Timeframe {
            start,
            end: start + chrono::Duration::minutes(window as i64),
        };

        let mut s = Self { frame, window };
        s.align();
        s
    }

    /// aligns the timeframe to the closest floor window multiple
    /// this will also clamp the end time to start+window
    pub fn align(&mut self) -> Option<()> {
        // SAFETY: the number is always valid
        let new_start = self.start().closest_floor(self.window)?;
        self.frame.start = new_start;
        self.frame.end = new_start + chrono::Duration::minutes(self.window as i64);
        Some(())
    }

    /// gets the previous timeframe
    /// end will be set to current start
    /// and start will be set start - window
    pub fn prev(&self) -> Self {
        let frame = Timeframe {
            start: *self.start() - chrono::Duration::minutes(self.window as i64),
            end: *self.start(),
        };

        Self {
            frame,
            window: self.window,
        }
    }

    /// get the next time frame
    /// makes start = end and end += window
    pub fn next(&self) -> Self {
        let frame = Timeframe {
            start: *self.end(),
            end: *self.end() + chrono::Duration::minutes(self.window as i64),
        };

        Self {
            frame,
            window: self.window,
        }
    }
}

#[test]
fn find_closest_window() {
    assert_eq!(closest_factor_of_60(9), 10);
    assert_eq!(closest_factor_of_60(14), 15);
    assert_eq!(closest_factor_of_60(12), 12);
    assert_eq!(closest_factor_of_60(13), 12);
    assert_eq!(closest_factor_of_60(17), 15); // both 15 and 20 result in 2.5 but 15 is returned since that appears 15 before 20 in the const array
}

#[test]
fn timeframe_tests() {
    let dt = Utc.with_ymd_and_hms(2014, 7, 8, 9, 10, 11).unwrap(); // `2014-07-08T09:10:11Z`

    // 60 min window test
    let window = 60;

    // Check 9:00-10:00 / 60 min window
    let base_start_time = Utc.with_ymd_and_hms(2014, 7, 8, 9, 0, 0).unwrap();
    let base_end_time = Utc.with_ymd_and_hms(2014, 7, 8, 10, 0, 0).unwrap();
    let even = EvenTimeframe::new(dt, window);

    assert_eq!(even.frame.start, base_start_time);
    assert_eq!(base_end_time, even.frame.end);

    // Check 10:00-11:00 / 60 min window
    let even = even.next();
    let base_start_time = Utc.with_ymd_and_hms(2014, 7, 8, 10, 0, 0).unwrap();
    let base_end_time = Utc.with_ymd_and_hms(2014, 7, 8, 11, 0, 0).unwrap();
    assert_eq!(even.frame.start, base_start_time);
    assert_eq!(base_end_time, even.frame.end);

    // 5 min window test
    let window = 5;

    // Check 9:10-9:15 / 5 min window
    let base_start_time = Utc.with_ymd_and_hms(2014, 7, 8, 9, 10, 0).unwrap();
    let base_end_time = Utc.with_ymd_and_hms(2014, 7, 8, 9, 15, 0).unwrap();
    let even = EvenTimeframe::new(dt, window);

    assert_eq!(even.frame.start, base_start_time);
    assert_eq!(base_end_time, even.frame.end);

    // Check 9:15-9:20 / 5 min window
    let even = even.next();
    let base_start_time = Utc.with_ymd_and_hms(2014, 7, 8, 9, 15, 0).unwrap();
    let base_end_time = Utc.with_ymd_and_hms(2014, 7, 8, 9, 20, 0).unwrap();
    assert_eq!(even.frame.start, base_start_time);
    assert_eq!(base_end_time, even.frame.end);

    // split test
    let window = 5;
    let start = Utc.with_ymd_and_hms(2014, 7, 8, 9, 15, 12).unwrap();
    let end = Utc.with_ymd_and_hms(2014, 7, 8, 21, 16, 12).unwrap();
    let chopped = EvenTimeframe::split(Timeframe { start, end }, window).unwrap();
    let first_res = EvenTimeframe::new(start, window);
    assert_eq!(*chopped.first().unwrap(), first_res);
    let last_res = EvenTimeframe::new(end, window);
    assert_eq!(last_res, *chopped.last().unwrap())
}
