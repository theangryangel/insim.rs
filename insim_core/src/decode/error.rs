use std::borrow::Cow;

#[derive(Debug, thiserror::Error)]
/// Decoding error
pub struct DecodeError {
    /// Optional contextual information
    pub context: Option<Cow<'static, str>>,
    /// Kind of error
    pub kind: DecodeErrorKind,
}

impl DecodeError {
    /// Add context to this error
    pub fn context(mut self, ctx: impl Into<Cow<'static, str>>) -> Self {
        self.context = Some(ctx.into());
        self
    }

    /// Create a nested error quickly
    pub fn nested(self) -> Self {
        Self {
            context: None,
            kind: DecodeErrorKind::Nested {
                source: Box::new(self),
            },
        }
    }
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut current: Option<&DecodeError> = Some(self);
        let mut first = true;

        while let Some(err) = current {
            if let Some(ctx) = &err.context {
                if !first {
                    f.write_str(" > ")?;
                }
                f.write_str(ctx)?;
                first = false;
            }

            match &err.kind {
                DecodeErrorKind::Nested { source } => current = Some(source),
                kind => {
                    if !first {
                        f.write_str(" > ")?;
                    }
                    write!(f, "{kind}")?;
                    break;
                },
            }
        }
        Ok(())
    }
}

impl From<DecodeErrorKind> for DecodeError {
    fn from(value: DecodeErrorKind) -> Self {
        Self {
            context: None,
            kind: value,
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
/// Kind of DecodeError
pub enum DecodeErrorKind {
    /// Expected more bytes
    #[error("Expected more bytes")]
    UnexpectedEof,

    /// Bad Magic
    #[error("Bad magic. Found: {:?}", found)]
    BadMagic {
        /// found
        found: Box<dyn core::fmt::Debug + Send + Sync>,
    },

    #[error("no variant match: {:?}", found)]
    /// No Variant
    NoVariantMatch {
        /// found
        found: u64,
    },

    /// Game Version Parse Error
    #[error("could not parse game version: {0}")]
    GameVersionParseError(#[from] crate::game_version::GameVersionParseError),

    /// Value too large or small for field
    #[error("Out of valid range: min: {min}, max: {max}, found: {found}")]
    OutOfRange {
        /// Minimum valid size
        min: usize,
        /// Maximum valid size
        max: usize,
        /// found
        found: usize,
    },

    /// Expected \0 character
    #[error("Expected \0 character")]
    ExpectedNull,

    /// Nested error - designed to preserve the full chain of errors
    #[error("{source}")]
    Nested {
        /// Source
        #[source]
        source: Box<DecodeError>,
    },
}

impl DecodeErrorKind {
    /// Add context to this error
    pub fn context(self, ctx: impl Into<Cow<'static, str>>) -> DecodeError {
        DecodeError {
            kind: self,
            context: Some(ctx.into()),
        }
    }
}

