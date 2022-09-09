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
    inputs: Vec<path::PathBuf>,

    #[clap(short, long)]
    output: path::PathBuf,

    #[clap(short, long, default_value_t = String::from("#3D9970"))]
    background_colour: String,

    #[clap(short, long, default_value_t = String::from("#111111"))]
    track_colour: String,
}

// FIXME - remove the unwraps
fn main() {
    let args = Args::parse();

    if args.output.exists() {
        // FIXME
        panic!("output already exists!");
    }

    let mut document = svg::Document::new();

    let mut all_pth_nodes = Vec::new();

    let mut x: (f32, f32) = (0.0, 0.0);
    let mut y: (f32, f32) = (0.0, 0.0);

    for i in args.inputs.iter() {
        if !i.exists() {
            // FIXME
            panic!("Could not find input file");
        }

        // FIXME
        let mut input = fs::File::open(i).unwrap();
        let mut buffer = Vec::new();

        // read the whole file
        input.read_to_end(&mut buffer).unwrap();

        let p = Pth::try_from(buffer.as_ref()).unwrap();

        // wrap around the nodes
        let mut nodes = p.nodes.clone();
        nodes.insert(0, *nodes.last().unwrap());

        all_pth_nodes.push(nodes);
    }

    // draw all the track limits first.
    for nodes in all_pth_nodes.iter() {
        for pair in nodes.windows(2) {
            let prev = pair[0];
            let next = pair[1];

            let prev_outer = prev.get_outer_limit(Some(65536.0));
            let next_outer = next.get_outer_limit(Some(65536.0));

            x.0 = x.0.min(prev_outer.0.x);
            x.1 = x.1.max(prev_outer.0.x);

            x.0 = x.0.min(prev_outer.1.x);
            x.1 = x.1.max(prev_outer.1.x);

            y.0 = y.0.min(prev_outer.0.y);
            y.1 = y.1.max(prev_outer.0.y);

            y.0 = y.0.min(prev_outer.1.y);
            y.1 = y.1.max(prev_outer.1.y);

            let poly = svg::node::element::Polygon::new()
                .set("style", format!("fill: {}", args.background_colour))
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
    }

    // draw all the roads next
    for nodes in all_pth_nodes.iter() {
        for pair in nodes.windows(2) {
            let prev = pair[0];
            let next = pair[1];

            let prev_outer = prev.get_road_limit(Some(65536.0));
            let next_outer = next.get_road_limit(Some(65536.0));

            let poly = svg::node::element::Polygon::new()
                .set("style", format!("fill: {}", args.track_colour))
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
    }

    document = document.set("viewBox", (x.0, y.0, x.1 - x.0, y.1 - y.0));

    svg::save(args.output, &document).unwrap();
}
