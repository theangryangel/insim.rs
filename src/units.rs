pub mod length {

    uom::unit! {
        system: uom::si;
        quantity: uom::si::length;

        // in LFS 65536 = 1 meter

        @game: 1.0/65536.0; "LFSu", "LFS world unit", "LFS world units";
    }
}

pub mod velocity {

    uom::unit! {
        system: uom::si;
        quantity: uom::si::velocity;

        // in LFS speed (32768 = 100 m/s)

        @game_per_second: 1.0/327.68; "LFSu/s", "LFS world unit per second", "LFS units per second";
    }
}

// TODO: Investigate. clippy seems to warn with the into() call, but without it the code wont compile.
#[allow(clippy::useless_conversion)]
pub mod angle {

    uom::unit! {
        system: uom::si;
        quantity: uom::si::angle;

        // NOTE 2) Heading : 0 = world y axis direction, 32768 = 180 degrees, anticlockwise from above

        @game_heading: (std::f32::consts::PI/32768.0).into(); "LFS°", "LFS degree", "LFS degrees";
    }
}

// TODO: Investigate. clippy seems to warn with the into() call, but without it the code wont compile.
#[allow(clippy::useless_conversion)]
pub mod angular_velocity {

    uom::unit! {
        system: uom::si;
        quantity: uom::si::angular_velocity;

        // NOTE 3) AngVel  : 0 = no change in heading,    8192 = 180 degrees per second anticlockwise

        @game_heading_per_second: (std::f32::consts::PI/8192.0).into(); "LFS°/s", "LFS degree per second", "LFS degrees per second";
    }
}
