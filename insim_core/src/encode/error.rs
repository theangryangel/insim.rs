use std::borrow::Cow;

#[derive(Debug, thiserror::Error)]
/// Encoding Error
pub struct EncodeError {
    /// Optional contextual information
    pub context: Option<Cow<'static, str>>,
    /// Kind of error
    pub kind: EncodeErrorKind,
}

impl EncodeError {
    /// Add context to this error
    pub fn context(mut self, ctx: impl Into<Cow<'static, str>>) -> Self {
        self.context = Some(ctx.into());
        self
    }

    /// Create a nested error quickly
    pub fn nested(self) -> Self {
        Self {
            context: None,
            kind: EncodeErrorKind::Nested {
                source: Box::new(self),
            },
        }
    }
}

impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut current: Option<&EncodeError> = Some(self);
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
                EncodeErrorKind::Nested { source } => current = Some(source),
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

impl From<EncodeErrorKind> for EncodeError {
    fn from(value: EncodeErrorKind) -> Self {
        Self {
            context: None,
            kind: value,
        }
    }
}

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
/// Kind of EncodeError
pub enum EncodeErrorKind {
    #[error("No variant match: {:?}", found)]
    /// No Variant
    NoVariantMatch {
        /// found
        found: u64,
    },

    /// String is not completely Ascii
    #[error("Not an ascii string")]
    NotAsciiString,

    /// Cannot convert
    #[error("Not an ascii char: {:?}", found)]
    NotAsciiChar {
        /// Found character
        found: char,
    },

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

    /// Nested error - designed to preserve the full chain of errors
    #[error("{source}")]
    Nested {
        /// Source
        #[source]
        source: Box<EncodeError>,
    },
}

impl EncodeErrorKind {
    /// Add context to this error
    pub fn context(self, ctx: impl Into<Cow<'static, str>>) -> EncodeError {
        EncodeError {
            kind: self,
            context: Some(ctx.into()),
        }
    }
}
