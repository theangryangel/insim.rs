use core::fmt;

use super::InsimString;

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

impl fmt::Display for InsimString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl fmt::Debug for InsimString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
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

// TODO add From InsimString into String, etc.
