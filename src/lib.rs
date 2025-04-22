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

impl<Tz: TimeZone> Timeframe<Tz> {
    /// convert into an even timeframe
    pub fn into_even(self, window: TimeWindow) -> Option<EvenTimeframe<Tz>> {
        EvenTimeframe::new(self.start, window)
    }
}

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

impl<Tz: TimeZone> Copy for EvenTimeframe<Tz> where
    <Tz as chrono::TimeZone>::Offset: std::marker::Copy
{
}

/// Easy Access functions
impl<Tz: TimeZone> EvenTimeframe<Tz> {
    pub fn start(&self) -> &DateTime<Tz> {
        &self.frame.start
    }

    pub fn end(&self) -> &DateTime<Tz> {
        &self.frame.end
    }
}

impl<Tz: TimeZone> EvenTimeframe<Tz> {
    /// Chops up the given timeframe after flooring the start time
    /// and ceiling the end time based on the provided window
    pub fn split(mut frame: Timeframe<Tz>, mut window: TimeWindow) -> Result<Vec<Self>, TimeErr> {
        window = clamp_window(window).ok_or_else(|| {
            TimeErr::Other("An error occurred while clamping the window size. Ensure the window size is factor of 60.".into())
        })?;
        frame.start = frame.start.closest_floor(window).ok_or(TimeErr::Floor)?;
        frame.end = frame.end.closest_ceil(window).ok_or(TimeErr::Ceil)?;
        let delta = frame.end.clone() - frame.start.clone();

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
            let end = curr.clone() + chrono::Duration::minutes(window as i64);
            let fr = EvenTimeframe {
                frame: Timeframe {
                    start: curr,
                    end: end.clone(),
                },
                window,
            };

            curr = end;

            time_frames.push(fr);
        }

        Ok(time_frames)
    }

    /// time frame to adjust this will floor to closest window multiple.
    /// Expects window to be a factor 60
    pub fn new(start: DateTime<Tz>, window: TimeWindow) -> Option<Self> {
        let window = window.clamp(0, 60);

        if 60 % window != 0 {
            return None;
        }

        let frame = Timeframe {
            start: start.clone(),
            end: start + chrono::Duration::minutes(window as i64),
        };

        let mut s = Self { frame, window };
        s.align();
        s.into()
    }

    /// aligns the timeframe to the closest floor window multiple
    /// this will also clamp the end time to start+window
    pub fn align(&mut self) -> Option<()> {
        // SAFETY: the number is always valid
        let new_start = self.start().closest_floor(self.window)?;
        self.frame.start = new_start.clone();
        self.frame.end = new_start + chrono::Duration::minutes(self.window as i64);
        Some(())
    }

    /// get the next time frame
    /// makes start = end and end += window
    pub fn next(&self) -> Self {
        let frame = Timeframe {
            start: self.end().clone(),
            end: self.end().clone() + chrono::Duration::minutes(self.window as i64),
        };

        Self {
            frame,
            window: self.window,
        }
    }
}

#[test]
fn timeframe_tests() {
    let dt = Utc.with_ymd_and_hms(2014, 7, 8, 9, 10, 11).unwrap(); // `2014-07-08T09:10:11Z`

    // 60 min window test
    let window = 60;

    // Check 9:00-10:00 / 60 min window
    let base_start_time = Utc.with_ymd_and_hms(2014, 7, 8, 9, 0, 0).unwrap();
    let base_end_time = Utc.with_ymd_and_hms(2014, 7, 8, 10, 0, 0).unwrap();
    let even = EvenTimeframe::new(dt, window).unwrap();

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
    let even = EvenTimeframe::new(dt, window).unwrap();

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
    let first_res = EvenTimeframe::new(start, window).unwrap();
    assert_eq!(*chopped.first().unwrap(), first_res);
    let last_res = EvenTimeframe::new(end, window).unwrap();
    assert_eq!(last_res, *chopped.last().unwrap())
}
