//! Utilities for positional and directional information in 3D space
macro_rules! impl_encode_decode_for_glam_vec3 {
    ($outer_type:path, $inner_type:ident) => {
        impl crate::Decode for $outer_type {
            fn decode(buf: &mut ::bytes::Bytes) -> Result<Self, crate::DecodeError> {
                let x = $inner_type::decode(buf)?;
                let y = $inner_type::decode(buf)?;
                let z = $inner_type::decode(buf)?;
                return Ok(Self { x, y, z });
            }
        }

        impl crate::Encode for $outer_type {
            fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), crate::EncodeError> {
                self.x.encode(buf)?;
                self.y.encode(buf)?;
                self.z.encode(buf)?;
                Ok(())
            }
        }
    };
}

impl_encode_decode_for_glam_vec3!(::glam::Vec3, f32);
impl_encode_decode_for_glam_vec3!(::glam::IVec3, i32);
