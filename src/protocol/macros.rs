#[macro_export]
macro_rules! generate_insim_packet {
    (
        $name: ident,

        $($variant:ident => $inner:ty, $id:literal,)+
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

