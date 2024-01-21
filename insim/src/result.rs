/// A `Result` alias where the `Err` case is `insim::Error`.
pub type Result<T> = std::result::Result<T, crate::error::Error>;
