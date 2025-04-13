macro_rules! impl_bitflags_from_to_bytes {
    ($type:ident, $inner_type:ident) => {
        impl ::insim_core::ReadWriteBuf for $type {
            fn read_buf(buf: &mut ::bytes::Bytes) -> Result<Self, insim_core::Error> {
                let inner = $inner_type::read_buf(buf)?;
                return Ok(Self::from_bits_truncate(inner));
            }

            fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
                let bits = <$type as ::bitflags::Flags>::bits(self);
                bits.write_buf(buf)?;
                Ok(())
            }
        }
    };
}
