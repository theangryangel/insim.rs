use bytes::Buf;

use crate::hex::HexDisplay;

/// DecodeContext
#[derive(Debug)]
pub struct DecodeContext<'a> {
    /// The underlying buffer being decoded
    pub buf: &'a mut bytes::Bytes,
}

impl<'a> DecodeContext<'a> {
    /// New
    pub fn new(buf: &'a mut bytes::Bytes) -> Self {
        Self { buf }
    }

    /// The core execution wrapper. Handles:
    /// Naming the field in a tracing span.
    /// Capturing bytes for a tracing dump
    /// Mapping errors with breadcrumbs
    fn op<T, F>(&mut self, name: &'static str, is_prim: bool, f: F) -> Result<T, super::DecodeError>
    where
        F: FnOnce(&mut Self) -> Result<T, super::DecodeError>,
    {
        // Enter a span. Tracing will handle the "Parent > Child" indentation automatically.
        let span = if is_prim {
            tracing::Span::none()
        } else {
            tracing::trace_span!("decode", field = name)
        };
        let _entered = span.entered();

        // Snapshot the buffer state for the HexDump
        let start_buf = if tracing::enabled!(tracing::Level::TRACE) {
            Some(self.buf.clone())
        } else {
            None
        };

        // Execute the actual decode logic
        let result = f(self).map_err(|e| e.nested().context(name))?;

        // Log the consumed bytes
        if let Some(start) = start_buf {
            let consumed = start.len() - self.buf.len();
            if consumed > 0 {
                let slice = start.slice(..consumed);
                let display_bytes = HexDisplay(&slice);
                if is_prim {
                    tracing::trace!(field = name, bytes = %display_bytes, "read");
                } else {
                    tracing::trace!(bytes = %display_bytes);
                }
            }
        }

        Ok(result)
    }

    /// Read any type that implements our basic Decode trait
    pub fn decode<T: super::Decode>(
        &mut self,
        name: &'static str,
    ) -> Result<T, super::DecodeError> {
        self.op(name, T::PRIMITIVE, |reader| T::decode(reader))
    }

    /// Paddding
    pub fn pad(&mut self, name: &'static str, len: usize) -> Result<(), super::DecodeError> {
        self.op(name, true, |reader| {
            if reader.buf.remaining() < len {
                return Err(super::DecodeErrorKind::UnexpectedEof.into());
            }
            reader.buf.advance(len);
            Ok(())
        })
    }

    /// Read a primitive integer and convert it to a [std::time::Duration] in milliseconds.
    pub fn decode_duration<T>(
        &mut self,
        name: &'static str,
    ) -> Result<std::time::Duration, super::DecodeError>
    where
        T: super::Decode + num_traits::ToPrimitive,
    {
        self.op(name, true, |ctx| {
            let raw = T::decode(ctx)?;
            raw.to_u64()
                .map(std::time::Duration::from_millis)
                .ok_or_else(|| {
                    super::DecodeErrorKind::OutOfRange {
                        min: 0,
                        max: u64::MAX as usize,
                        found: raw.to_usize().unwrap_or(usize::MAX),
                    }
                    .into()
                })
        })
    }

    /// Special case: fixed length codepage string
    pub fn decode_codepage(
        &mut self,
        name: &'static str,
        len: usize,
    ) -> Result<String, super::DecodeError> {
        self.op(name, true, |reader| {
            let new = reader.buf.copy_to_bytes(reader.buf.len().min(len));
            let new =
                crate::string::codepages::to_lossy_string(crate::string::strip_trailing_nul(&new));
            Ok(new.to_string())
        })
    }

    /// Special case: fixed length ascii string
    pub fn decode_ascii(
        &mut self,
        name: &'static str,
        len: usize,
    ) -> Result<String, super::DecodeError> {
        self.op(name, true, |reader| {
            if reader.buf.remaining() < len {
                return Err(super::DecodeErrorKind::UnexpectedEof.into());
            }
            let new = reader.buf.copy_to_bytes(len);
            let bytes = crate::string::strip_trailing_nul(&new);
            Ok(String::from_utf8_lossy(bytes).into_owned())
        })
    }
}
