macro_rules! generate_bitflag_helpers {
    ($struct_name:ident $(, $vis:vis $method:ident => $flag:ident)*) => {

        impl $struct_name {
            $(
                    #[allow(unreachable_pub)]
                    #[doc="Convenience method to check if "]
                    #[doc=stringify!($flag)]
                    #[doc=" flag is set."]
                    $vis fn $method(&self) -> bool {
                        self.contains(Self::$flag)
                    }
            )*
        }
    };
}

#[cfg(test)]
mod test {
    bitflags::bitflags! {
        struct MyFlags: u32 {
            const FLAG_A = 0b00000001;
            const FLAG_B = 0b00000010;
            const FLAG_C = 0b00000100;
            const FLAG_D = 0b00001000;
        }
    }

    generate_bitflag_helpers!(
        MyFlags,
        pub(super) is_flag_a => FLAG_A,
        pub is_flag_b => FLAG_B,
        is_flag_c => FLAG_C
    );

    #[test]
    fn name() {
        let flags = MyFlags::FLAG_B | MyFlags::FLAG_C;

        assert!(!flags.is_flag_a());
        assert!(flags.is_flag_b());
        assert!(flags.is_flag_c());
    }
}
