#[macro_export]
macro_rules! packet {
    (
        $name: ident,

        $($id:literal => $variant:ident($inner:ty),)+
    ) => {

        // Create the enum itself
        #[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
        #[serde(tag = "type")]
        #[deku(endian = "little", type = "u8")]
        pub enum $name {
            $(
                #[deku(id = $id)]
                $variant($inner)
            ),+
        }

        impl $name {
            // Allow us to get the packet name from the variant
            pub fn name(&self) -> &str {
                match &self {
                    $($name::$variant{..} => stringify!($variant),)+
                }
            }
        }

        // Implement From for all our variants so that we can use do insim::Init().into() to get a
        // Packet::Init(..) variant
        $(
        impl From<$inner> for $name {
            fn from(item: $inner) -> Self {
                Self::$variant(item)
            }
        }
        )+
    }
}

#[macro_export]
macro_rules! packet_flags {
    (
        $(#[$outer:meta])*
        $vis:vis struct $name: ident: $T:ty {
            $(
                $flag:ident => $value:expr,
            )*
        }
    ) => {

        ::bitflags::bitflags! {
            $(#[$outer])*
            #[derive(Default)]
            $vis struct $name: $T {
                $(
                    const $flag = $value;
                )*
            }
        }

        #[allow(unused)]
        impl $name {
            pub fn clear(&mut self) {
                self.bits = 0;
            }
        }

        impl ::deku::DekuWrite<(::deku::ctx::Endian, ::deku::ctx::Size)> for $name {
            fn write(
                &self,
                output: &mut ::deku::bitvec::BitVec<::deku::bitvec::Msb0, u8>,
                (_endian, _bit_size): (::deku::ctx::Endian, ::deku::ctx::Size),
            ) -> Result<(), ::deku::DekuError> {
                // FIXME: implement endian and size limits
                let value = self.bits();
                value.write(output, ())
            }
        }

        impl ::deku::DekuWrite<::deku::ctx::Size> for $name {
            fn write(
                &self,
                output: &mut ::deku::bitvec::BitVec<::deku::bitvec::Msb0, u8>,
                _bit_size: ::deku::ctx::Size
            ) -> Result<(), ::deku::DekuError> {
                // FIXME: implement size limits
                let value = self.bits();
                value.write(output, ())
            }
        }

        impl ::deku::DekuWrite for $name {
            fn write(
                &self, output:
                &mut ::deku::bitvec::BitVec<::deku::bitvec::Msb0, u8>, _: ()
            ) -> Result<(), ::deku::DekuError> {
                let value = self.bits();
                value.write(output, ())
            }
        }

        impl ::deku::DekuRead<'_, ::deku::ctx::Size> for $name {
            fn read(
                input: &::deku::bitvec::BitSlice<::deku::bitvec::Msb0, u8>,
                size: ::deku::ctx::Size,
            ) -> Result<(&::deku::bitvec::BitSlice<::deku::bitvec::Msb0, u8>, Self), ::deku::DekuError> {
                let endian = ::deku::ctx::Endian::default();
                let (rest, value) = <$T>::read(input, (endian, size))?;

                Ok((
                    rest,
                    $name::from_bits_truncate(value)
                ))
            }
        }

        impl ::deku::DekuRead<'_, (::deku::ctx::Endian, ::deku::ctx::Size)> for $name {
            fn read(
                input: &::deku::bitvec::BitSlice<::deku::bitvec::Msb0, u8>,
                (endian, size): (::deku::ctx::Endian, ::deku::ctx::Size),
            ) -> Result<(&::deku::bitvec::BitSlice<::deku::bitvec::Msb0, u8>, Self), ::deku::DekuError> {
                let (rest, value) = <$T>::read(input, (endian, size))?;

                Ok((
                    rest,
                    $name::from_bits_truncate(value)
                ))
            }
        }

        impl ::deku::DekuRead<'_> for $name {
            fn read(
                input: &::deku::bitvec::BitSlice<::deku::bitvec::Msb0, u8>,
                _: (),
            ) -> Result<(&::deku::bitvec::BitSlice<::deku::bitvec::Msb0, u8>, Self), ::deku::DekuError> {
                let (rest, value) = <$T>::read(input, ::deku::ctx::Endian::default())?;

                Ok((
                    rest,
                    $name::from_bits_truncate(value)
                ))
            }
        }
    }
}
