macro_rules! impl_bitflags_from_to_bytes {
    ($type:ident, $inner_type:ident) => {
        impl ::insim_core::Decode for $type {
            fn decode(ctx: &mut ::insim_core::DecodeContext) -> Result<Self, insim_core::DecodeError> {
                ctx.decode::<$inner_type>("bits").map(Self::from_bits_truncate)
            }
        }

        impl ::insim_core::Encode for $type {
            fn encode(&self, ctx: &mut ::insim_core::EncodeContext) -> Result<(), insim_core::EncodeError> {
                let bits = <$type as ::bitflags::Flags>::bits(self);
                ctx.encode("bits", &bits)
            }
        }
    };
}
