use super::IString;
use std::fmt;

impl Clone for IString {
    fn clone(&self) -> Self {
        IString {
            inner: self.inner.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.inner.clone_from(&source.inner);
    }
}

impl From<String> for IString {
    #[inline]
    fn from(s: String) -> Self {
        IString::from_string(s)
    }
}

impl From<&str> for IString {
    #[inline]
    fn from(s: &str) -> Self {
        IString::from_string(s.into())
    }
}

impl fmt::Display for IString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_lossy_string())
    }
}
