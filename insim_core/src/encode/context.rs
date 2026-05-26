use bytes::BufMut;

use crate::hex::HexDisplay;

/// EncodeContext
#[derive(Debug)]
pub struct EncodeContext<'a> {
    /// The underlying buffer being encoded into
    pub buf: &'a mut bytes::BytesMut,
}

impl<'a> EncodeContext<'a> {
    /// New
    pub fn new(buf: &'a mut bytes::BytesMut) -> Self {
        Self { buf }
    }

    fn op<F>(&mut self, name: &'static str, is_prim: bool, f: F) -> Result<(), super::EncodeError>
    where
        F: FnOnce(&mut Self) -> Result<(), super::EncodeError>,
    {
        let span = if is_prim {
            tracing::Span::none()
        } else {
            tracing::trace_span!("encode", field = name)
        };
        let _entered = span.entered();
        let start_len = self.buf.len();

        f(self).map_err(|e| e.nested().context(name))?;

        if tracing::enabled!(tracing::Level::TRACE) {
            let written = &self.buf[start_len..];
            if !written.is_empty() {
                let display_bytes = HexDisplay(written);
                if is_prim {
                    tracing::trace!(field = name, bytes = %display_bytes, "wrote");
                } else {
                    tracing::trace!(bytes = %display_bytes);
                }
            }
        }
        Ok(())
    }

    /// Paddding
    pub fn pad(&mut self, name: &'static str, len: usize) -> Result<(), super::EncodeError> {
        self.op(name, true, |w| {
            w.buf.put_bytes(0, len);
            Ok(())
        })
    }

    /// Encode anything that impls Encode
    pub fn encode<T: super::Encode>(
        &mut self,
        name: &'static str,
        val: &T,
    ) -> Result<(), super::EncodeError> {
        self.op(name, T::PRIMITIVE, |writer| val.encode(writer))
    }

    /// Convert a [std::time::Duration] to milliseconds and encode it as a primitive integer.
    pub fn encode_duration<T>(
        &mut self,
        name: &'static str,
        val: std::time::Duration,
    ) -> Result<(), super::EncodeError>
    where
        T: super::Encode + num_traits::NumCast + num_traits::Bounded,
    {
        self.op(name, true, |ctx| {
            let millis = val.as_millis();
            let max = num_traits::cast::<T, usize>(T::max_value()).unwrap_or(usize::MAX);
            match num_traits::cast::<u128, T>(millis) {
                Some(v) => v.encode(ctx),
                None => Err(super::EncodeErrorKind::OutOfRange {
                    min: 0,
                    max,
                    found: millis as usize,
                }
                .into()),
            }
        })
    }

    /// Write
    pub fn encode_ascii<T: AsRef<str>>(
        &mut self,
        name: &'static str,
        val: T,
        len: usize,
        trailing_nul: bool,
    ) -> Result<(), super::EncodeError> {
        if !val.as_ref().is_ascii() {
            return Err(super::EncodeErrorKind::NotAsciiString.into());
        }

        self.op(name, true, |writer| {
            let new = val.as_ref().as_bytes();
            let max_len = if trailing_nul { len - 1 } else { len };
            if new.len() > max_len {
                return Err(super::EncodeErrorKind::OutOfRange {
                    min: 0,
                    max: max_len,
                    found: new.len(),
                }
                .into());
            }
            let len_to_write = new.len().min(max_len);
            writer.buf.extend_from_slice(&new[..len_to_write]);
            writer.buf.put_bytes(0, len - len_to_write);
            Ok(())
        })
    }

    /// Write a codepage string. If `align_to` is `Some`, `len` is the maximum length and the
    /// written bytes are padded to the nearest `align_to`-byte boundary. If `align_to` is `None`,
    /// `len` is the exact fixed length and the field is always padded to exactly `len` bytes.
    pub fn encode_codepage<T: AsRef<str>>(
        &mut self,
        name: &'static str,
        val: T,
        len: usize,
        align_to: Option<usize>,
        trailing_nul: bool,
    ) -> Result<(), super::EncodeError> {
        self.op(name, true, |writer| {
            let new = crate::string::codepages::to_lossy_bytes(val.as_ref());
            let max_len = if trailing_nul { len - 1 } else { len };
            if new.len() > max_len {
                return Err(super::EncodeErrorKind::OutOfRange {
                    min: 0,
                    max: max_len,
                    found: new.len(),
                }
                .into());
            }
            let len_to_write = new.len().min(max_len);
            writer.buf.extend_from_slice(&new[..len_to_write]);

            if let Some(alignment) = align_to {
                let mask = alignment - 1;
                let min_total = if trailing_nul {
                    len_to_write + 1
                } else {
                    len_to_write
                };
                let round_to = ((min_total + mask) & !mask).min(len);
                if round_to > len_to_write {
                    writer.buf.put_bytes(0, round_to - len_to_write);
                }
            } else {
                writer.buf.put_bytes(0, len - len_to_write);
            }
            Ok(())
        })
    }

    /// Write an optional codepage string. See encode_codepage for a breakdown of args.
    pub fn encode_optional_codepage<T: AsRef<str>>(
        &mut self,
        name: &'static str,
        val: &Option<T>,
        len: usize,
        align_to: Option<usize>,
        trailing_nul: bool,
    ) -> Result<(), super::EncodeError> {
        match val {
            None => self.encode_codepage(name, "", len, align_to, trailing_nul),
            Some(v) => self.encode_codepage(name, v.as_ref(), len, align_to, trailing_nul),
        }
    }
}
