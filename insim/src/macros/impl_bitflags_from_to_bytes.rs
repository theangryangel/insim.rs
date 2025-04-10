macro_rules! impl_bitflags_from_to_bytes {
    ($type:ident, $inner_type:ident) => {
        impl ::insim_core::FromToBytes for $type {
            fn from_bytes(buf: &mut ::bytes::Bytes) -> Result<Self, insim_core::Error> {
                let inner = $inner_type::from_bytes(buf)?;
                return Ok(Self::from_bits_truncate(inner));
            }

            fn to_bytes(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
                let bits = <$type as ::bitflags::Flags>::bits(self);
                bits.to_bytes(buf)?;
                Ok(())
            }
        }
    };
}
