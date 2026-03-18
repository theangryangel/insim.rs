use bytes::Buf;

/// DecodeContext
#[derive(Debug)]
pub struct DecodeContext<'a> {
    pub(crate) buf: &'a mut bytes::Bytes,
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
    fn op<T, F>(&mut self, name: &'static str, f: F) -> Result<T, super::DecodeError>
    where
        F: FnOnce(&mut Self) -> Result<T, super::DecodeError>,
    {
        // Enter a span. Tracing will handle the "Parent > Child" indentation automatically.
        let _span = tracing::trace_span!("decode", field = name).entered();
        
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
                tracing::trace!(bytes = ?&start.slice(..consumed), "ok");
            }
        }

        Ok(result)
    }

    /// Read any type that implements our basic Decode trait
    pub fn decode<T: super::Decode>(&mut self, name: &'static str) -> Result<T, super::DecodeError> {
        self.op(name, |reader| T::decode(reader))
    }

    /// Paddding
    pub fn pad(&mut self, name: &'static str, len: usize) -> Result<(), super::DecodeError> {
        self.op(name, |reader| {
            if reader.buf.remaining() < len {
                return Err(super::DecodeErrorKind::UnexpectedEof.into());

            }
            reader.buf.advance(len);
            Ok(())
        })
    }

    /// Special case: fixed length codepage string
    pub fn decode_codepage(&mut self, name: &'static str, len: usize) -> Result<String, super::DecodeError> {
        self.op(name, |reader| {
            let new = reader.buf.copy_to_bytes(reader.buf.len().min(len));
            let new =
                crate::string::codepages::to_lossy_string(crate::string::strip_trailing_nul(&new));
            Ok(new.to_string())
        })
    }

    /// Special case: fixed length ascii string
    pub fn decode_ascii(&mut self, name: &'static str, len: usize) -> Result<String, super::DecodeError> {
        self.op(name, |reader| {
            if reader.buf.remaining() < len {
                return Err(super::DecodeErrorKind::UnexpectedEof.into());
            }
            let new = reader.buf.copy_to_bytes(len);
            let bytes = crate::string::strip_trailing_nul(&new);
            Ok(String::from_utf8_lossy(bytes).to_string())
        })
    }
}
