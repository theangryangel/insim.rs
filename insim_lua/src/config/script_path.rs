use std::default::Default;
use std::{
    io,
    ops::Deref,
    path::{Path, PathBuf},
};

#[derive(Debug, Default)]
pub(crate) struct ScriptPath {
    pub(crate) inner: PathBuf,
}

impl Deref for ScriptPath {
    type Target = PathBuf;
    fn deref(&self) -> &PathBuf {
        &self.inner
    }
}

impl From<String> for ScriptPath {
    #[inline]
    fn from(s: String) -> ScriptPath {
        Self {
            inner: PathBuf::from(s),
        }
    }
}

impl AsRef<Path> for ScriptPath {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.inner.as_ref()
    }
}

impl<S: knuffel::traits::ErrorSpan> knuffel::traits::DecodeScalar<S> for ScriptPath {
    fn raw_decode(
        val: &knuffel::span::Spanned<knuffel::ast::Literal, S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<ScriptPath, knuffel::errors::DecodeError<S>> {
        match &**val {
            knuffel::ast::Literal::String(ref s) => {
                let buf: Self = String::from(s.clone()).into();
                if !buf.exists() {
                    ctx.emit_error(knuffel::errors::DecodeError::conversion(
                        val,
                        io::Error::new(io::ErrorKind::NotFound, "Path does not exist"),
                    ));
                    Ok(Default::default())
                } else {
                    Ok(buf)
                }
            }
            _ => {
                ctx.emit_error(knuffel::errors::DecodeError::scalar_kind(
                    knuffel::decode::Kind::String,
                    val,
                ));
                Ok(Default::default())
            }
        }
    }

    fn type_check(
        type_name: &Option<knuffel::span::Spanned<knuffel::ast::TypeName, S>>,
        ctx: &mut knuffel::decode::Context<S>,
    ) {
        if let Some(typ) = type_name {
            ctx.emit_error(knuffel::errors::DecodeError::TypeName {
                span: typ.span().clone(),
                found: Some(typ.deref().clone()),
                expected: knuffel::errors::ExpectedType::no_type(),
                rust_type: "String",
            });
        }
    }
}
