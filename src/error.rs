use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TimeErr {
    /// Unable to floor the time
    /// to a given window
    #[error("An error occurred while flooring the time")]
    Floor,
    /// Unable to ceil the time
    /// to a given window
    #[error("An error occurred while Ceiling the time")]
    Ceil,
    /// Timeframe is too large
    #[error(
        "Provided timeframe is too large to process. Try reducing the timeframe to fewer days/ weeks"
    )]
    FrameTooLarge,
    /// Custom
    #[error("{0}")]
    Other(String),
}
