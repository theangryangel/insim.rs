use bytes::{BytesMut, Bytes};
use crate::result::Result;

use super::{codec::Codec, Framed};

#[async_trait::async_trait]
pub trait Transport: Sized {
    async fn read(&mut self, buf: &mut BytesMut) -> Result<usize>;
    async fn write(&mut self, src: &mut BytesMut) -> Result<()>;
}

pub trait IntoFramed: Transport {
    fn into_framed<C: Codec>(self, codec: C) -> Framed<C, Self> {
        Framed::new(self, codec)       
    }
}

