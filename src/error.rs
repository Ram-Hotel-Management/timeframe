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
