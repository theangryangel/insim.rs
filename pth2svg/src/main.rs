use clap::Parser;
use insim::file::pth::Pth;
use std::fs;
use std::io::Read;
use std::path;

/// pth2svg does stuff
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    input: path::PathBuf,

    #[clap(short, long)]
    output: path::PathBuf,
}

// FIXME - remove the unwraps
fn main() {
    let args = Args::parse();

    if !args.input.exists() {
        // FIXME
        panic!("Could not find input file");
    }

    if args.output.exists() {
        // FIXME
        panic!("output already exists!");
    }

    // FIXME
    let mut input = fs::File::open(args.input).unwrap();

    let mut buffer = Vec::new();

    // read the whole file
    input.read_to_end(&mut buffer).unwrap();

    let p = Pth::try_from(buffer.as_ref()).unwrap();

    //let mut outer = Vec::new();
    //let mut road = Vec::new();

    let mut x: (f32, f32) = (0.0, 0.0);
    let mut y: (f32, f32) = (0.0, 0.0);

    let mut nodes = p.nodes;
    nodes.insert(0, *nodes.last().unwrap());

    let mut document = svg::Document::new();

    let mut data = svg::node::element::path::Data::new().move_to((
        nodes.first().unwrap().center.x as f32 / 65536.0,
        nodes.first().unwrap().center.y as f32 / 65536.0,
    ));

    for pair in nodes.windows(2) {
        let prev = pair[0];
        let next = pair[1];

        let prev_outer = prev.get_outer_limit(Some(65536.0));
        let next_outer = next.get_outer_limit(Some(65536.0));

        data = data.line_to((
            prev.center.x as f32 / 65536.0,
            prev.center.y as f32 / 65536.0,
        ));

        x.0 = x.0.min(prev_outer.0.x);
        x.1 = x.1.max(prev_outer.0.x);

        x.0 = x.0.min(prev_outer.1.x);
        x.1 = x.1.max(prev_outer.1.x);

        y.0 = y.0.min(prev_outer.0.y);
        y.1 = y.1.max(prev_outer.0.y);

        y.0 = y.0.min(prev_outer.1.y);
        y.1 = y.1.max(prev_outer.1.y);

        let poly = svg::node::element::Polygon::new()
            .set("style", "fill: green")
            .set(
                "points",
                format!(
                    "{},{} {},{} {},{} {},{}",
                    prev_outer.0.x,
                    prev_outer.0.y,
                    next_outer.0.x,
                    next_outer.0.y,
                    next_outer.1.x,
                    next_outer.1.y,
                    prev_outer.1.x,
                    prev_outer.1.y,
                ),
            );

        document = document.add(poly);

        let prev_outer = prev.get_road_limit(Some(65536.0));
        let next_outer = next.get_road_limit(Some(65536.0));

        let poly = svg::node::element::Polygon::new()
            .set("style", "fill: black")
            .set(
                "points",
                format!(
                    "{},{} {},{} {},{} {},{}",
                    prev_outer.0.x,
                    prev_outer.0.y,
                    next_outer.0.x,
                    next_outer.0.y,
                    next_outer.1.x,
                    next_outer.1.y,
                    prev_outer.1.x,
                    prev_outer.1.y,
                ),
            );

        document = document.add(poly);
    }

    document = document.set("viewBox", (x.0, y.0, x.1 - x.0, y.1 - y.0));

    data = data.close();

    let path = svg::node::element::Path::new()
        .set("fill", "none")
        .set("stroke", "red")
        .set("stroke-width", 1)
        .set("d", data);

    document = document.add(path);

    svg::save(args.output, &document).unwrap();
}
