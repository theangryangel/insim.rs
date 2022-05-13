#[macro_export]
macro_rules! packet_id {
    ($id:literal) => {
        stringify!($id)
    };
}

/// Internal macro to define Insim Packet and conversion implementations.
#[macro_export]
macro_rules! packet {
    (
        $name: ident,

        $(
            $id:literal => $variant:ident($inner:ty),
        )+
    ) => {
        use std::str::FromStr;

        /// Enum of all possible packet types.
        #[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
        #[cfg_attr(feature = "serde", derive(Serialize))]
        #[cfg_attr(feature = "serde", serde(tag = "type"))]
        #[deku(endian = "little", type = "u8")]
        pub enum $name {
            $(
                #[deku(id = $id)]
                $variant($inner)
            ),+
        }

        impl $name {
            // Allow us to get the packet name from the variant
            pub fn inner_name(&self) -> &str {
                match &self {
                    $($name::$variant{..} => stringify!($inner),)+
                }
            }

            // Allow us to get the variant name
            pub fn name(&self) -> &str {
                match &self {
                    $($name::$variant{..} => stringify!($variant),)+
                }
            }

            // Return the numerical id of the packet
            pub fn id(&self) -> u8 {
                match &self {
                    // TODO: hoping the compiler is smart enough to inline this.
                    // Lets find out.
                    // Is there a secret Deku fn we can use instead?
                    $($name::$variant{..} => u8::from_str($id).unwrap(),)+
                }
            }

            // Convert a name into the numeric id of the packet
            pub fn name_into_id(input: &str) -> Option<u8> {
                match input {
                    // TODO: See above notes
                    $(stringify!($variant) => Some(u8::from_str($id).unwrap()),)+
                    _ => { None }
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

        // #[cfg(feature = "client")]
        // $(
        // impl From<$inner> for crate::client::Command {
        //     fn from(item: $inner) -> Self {
        //         crate::client::Command::Frame(
        //             $name::$variant(item)
        //         )
        //     }
        // }
        // )+

    }
}

/// Internal macro to help define packet bit-wise flags and provide a high level API to ensure you
/// don't accidentally set bits that are not intended to be set.
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
