use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeErr {
    /// Unable to floor the time
    /// to a given window
    Floor,
    /// Unable to ceil the time
    /// to a given window
    Ceil,
    /// Timeframe is too large
    FrameTooLarge,
    /// Custom
    Other(String),
}

impl Display for TimeErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// impl std::error::Error for TimeErr {
//     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//         Some(self)
//     }

//     fn description(&self) -> &str {
//         "description() is deprecated; use Display"
//     }

//     fn cause(&self) -> Option<&dyn std::error::Error> {
//         self.source()
//     }
// }

impl std::error::Error for &TimeErr {}
