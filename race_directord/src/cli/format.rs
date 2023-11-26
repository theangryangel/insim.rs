#[derive(Clone, Default, Debug, clap::ValueEnum)]
pub(crate) enum Format {
    #[default]
    Compact,
    Full,
    Pretty,
    Json,
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let logger = match self {
            Format::Compact => "compact",
            Format::Full => "full",
            Format::Pretty => "pretty",
            Format::Json => "json",
        };
        write!(f, "{}", logger)
    }
}
