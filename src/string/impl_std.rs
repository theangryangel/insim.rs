use super::InsimString;
use std::fmt;

impl Clone for InsimString {
    fn clone(&self) -> Self {
        InsimString {
            inner: self.inner.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.inner.clone_from(&source.inner);
    }
}

impl From<String> for InsimString {
    #[inline]
    fn from(s: String) -> Self {
        InsimString::from_string(s)
    }
}

impl From<&str> for InsimString {
    #[inline]
    fn from(s: &str) -> Self {
        InsimString::from_string(s.into())
    }
}

impl fmt::Display for InsimString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_lossy_string())
    }
}
