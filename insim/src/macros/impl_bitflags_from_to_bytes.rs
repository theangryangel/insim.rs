macro_rules! impl_bitflags_from_to_bytes {
    ($type:ident, $inner_type:ident) => {
        impl ::insim_core::Decode for $type {
            fn decode(buf: &mut ::bytes::Bytes) -> Result<Self, insim_core::Error> {
                let inner = $inner_type::decode(buf)?;
                return Ok(Self::from_bits_truncate(inner));
            }
        }

        impl ::insim_core::Encode for $type {
            fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
                let bits = <$type as ::bitflags::Flags>::bits(self);
                bits.encode(buf)?;
                Ok(())
            }
        }
    };
}
