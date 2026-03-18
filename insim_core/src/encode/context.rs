use bytes::BufMut;

/// EncodeContext
#[derive(Debug)]
pub struct EncodeContext<'a> {
    pub(crate) buf: &'a mut bytes::BytesMut,
}

impl<'a> EncodeContext<'a> {
    /// New
    pub fn new(buf: &'a mut bytes::BytesMut) -> Self {
        Self { buf }
    }

    fn op<F>(&mut self, name: &'static str, f: F) -> Result<(), super::EncodeError>
    where
        F: FnOnce(&mut Self) -> Result<(), super::EncodeError>,
    {
        let _span = tracing::trace_span!("encode", field = name).entered();
        let start_len = self.buf.len();

        f(self).map_err(|e| e.nested().context(name))?;

        if tracing::enabled!(tracing::Level::TRACE) {
            let written = &self.buf[start_len..];
            if !written.is_empty() {
                tracing::trace!(bytes = ?written, "ok");
            }
        }
        Ok(())
    }

    /// Paddding
    pub fn pad(&mut self, name: &'static str, len: usize) -> Result<(), super::EncodeError> {
        self.op(name, |w| {
            w.buf.put_bytes(0, len);
            Ok(())
        })
    }

    /// Encode anything that impls Encode
    pub fn encode<T: super::Encode>(&mut self, name: &'static str, val: &T) -> Result<(), super::EncodeError> {
        self.op(name, |writer| val.encode(writer))
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

        self.op(name, |writer| {
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

    /// Write fixed length codepage
    pub fn encode_codepage<T: AsRef<str>>(
        &mut self,
        name: &'static str,
        val: T,
        len: usize,
        trailing_nul: bool,
    ) -> Result<(), super::EncodeError> {
        self.op(name, |writer| {
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
            writer.buf.put_bytes(0, len - len_to_write);
            Ok(())
        })
    }

    /// Write variable length codepage, upto len, aligned to nearest X bytes
    pub fn encode_codepage_with_alignment<T: AsRef<str>>(
        &mut self,
        name: &'static str,
        val: T,
        len: usize,
        alignment: usize,
        trailing_nul: bool,
    ) -> Result<(), super::EncodeError> {
        self.op(name, |writer| {
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

            // Always pad to alignment, ensuring trailing_nul if needed
            let align_to = alignment - 1;
            let min_total = if trailing_nul {
                len_to_write + 1
            } else {
                len_to_write
            };
            let round_to = (min_total + align_to) & !align_to;
            let round_to = round_to.min(len);

            if round_to > len_to_write {
                writer.buf.put_bytes(0, round_to - len_to_write);
            }
            Ok(())
        })
    }

}

