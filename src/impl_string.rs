use std::string::FromUtf8Error;
use std::io;

use crate::InsimString;

impl InsimString for String {

    fn from_lfs(value: Vec<u8>) -> Result<String, FromUtf8Error> {
        let i = value.iter().rposition(|x| *x != 0).unwrap();

        // TODO Handle encoding from codepages to utf-8
        // TODO This should probably be a custom type and just implement the deku read/write traits.

        String::from_utf8(value[..=i].to_vec())
    }

    fn to_lfs(&self, max_size: usize) -> Result<Vec<u8>, io::Error> {
        // TODO convert utf8 to codepages

        // TODO we can do this without allocating a buffer, etc.
        // Fix this.
        let mut buf = self.as_bytes().to_vec();
        if buf.len() < max_size {
            buf.reserve(max_size - buf.len());
            for _i in 0..(max_size-buf.len()) {
                buf.push(0);
            }
        }

        Ok(buf[0..max_size].to_vec())
    }
}
