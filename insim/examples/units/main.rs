extern crate insim;
use tracing_subscriber;
use uom;

fn setup() {
    // setup tracing with some defaults if nothing is set
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}

fn main() {
    setup();

    use insim::units::{
        angle::game_heading, angular_velocity::game_heading_per_second, length::game,
        velocity::game_per_second,
    };
    use uom::fmt::DisplayStyle::Abbreviation;
    use uom::si::angle::{degree, radian};
    use uom::si::angular_velocity::{degree_per_second, radian_per_second};
    use uom::si::length::{kilometer, meter, mile};
    use uom::si::velocity::{meter_per_second, mile_per_hour};

    let l1 = uom::si::f64::Length::new::<game>((65536 * 1000) as f64);
    let l2 = uom::si::f64::Length::new::<meter>(1.0);
    let l3 = uom::si::f64::Length::new::<kilometer>(1.0);

    println!(
        "{} = {} = {} = {}",
        l1.into_format_args(game, Abbreviation),
        l1.into_format_args(meter, Abbreviation),
        l1.into_format_args(kilometer, Abbreviation),
        l1.into_format_args(mile, Abbreviation),
    );
    println!(
        "{} = {} = {}",
        l2.into_format_args(meter, Abbreviation),
        l2.into_format_args(game, Abbreviation),
        l2.into_format_args(mile, Abbreviation),
    );

    println!(
        "{} = {}",
        l3.into_format_args(kilometer, Abbreviation),
        l3.into_format_args(game, Abbreviation),
    );

    let v1 = uom::si::f64::Velocity::new::<game_per_second>(32768.0);

    println!(
        "{} = {} = {}",
        v1.into_format_args(game_per_second, Abbreviation),
        v1.into_format_args(meter_per_second, Abbreviation),
        v1.into_format_args(mile_per_hour, Abbreviation),
    );

    let a1 = uom::si::f64::Angle::new::<game_heading>(32768 as f64);

    println!(
        "{} = {} = {}",
        a1.into_format_args(game_heading, Abbreviation),
        a1.into_format_args(degree, Abbreviation),
        a1.into_format_args(radian, Abbreviation),
    );

    let av1 = uom::si::f64::AngularVelocity::new::<game_heading_per_second>(16384 as f64);

    println!(
        "{} = {} = {}",
        av1.into_format_args(game_heading_per_second, Abbreviation),
        av1.into_format_args(degree_per_second, Abbreviation),
        av1.into_format_args(radian_per_second, Abbreviation),
    );
}
