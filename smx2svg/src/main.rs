use clap::Parser;
use insim::file::smx::Smx;
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
        panic!("Could not find smx file");
    }

    if args.output.exists() {
        // FIXME
        panic!("output already exists!");
    }

    // FIXME
    let mut input = fs::File::open(args.input).unwrap();

    let mut smx_buffer = Vec::new();

    // read the whole file
    input.read_to_end(&mut smx_buffer).unwrap();

    let smx = Smx::try_from(smx_buffer.as_ref()).unwrap();
    let nodes = smx.objects;

    let mut x: (f32, f32) = (0.0, 0.0);
    let mut y: (f32, f32) = (0.0, 0.0);

    let mut document = svg::Document::new();

    for node in nodes {
        for point in node.points.iter() {
            x.0 = x.0.min(point.xyz.x as f32 / 65536.0);
            x.1 = x.1.max(point.xyz.x as f32 / 65536.0);

            y.0 = y.0.min(point.xyz.y as f32 / 65536.0);
            y.1 = y.1.max(point.xyz.y as f32 / 65536.0);
        }

        for triangle in node.triangles.iter() {
            // FIXME: colours are wonky
            let colour = format!(
                "rgba({}, {}, {}, {})",
                node.points[triangle.a as usize].colour.rgb.r,
                node.points[triangle.a as usize].colour.rgb.g,
                node.points[triangle.a as usize].colour.rgb.b,
                node.points[triangle.a as usize].colour.a as f32 / 100.0,
            );

            let poly = svg::node::element::Polygon::new()
                .set("style", format!("fill: {}", colour))
                .set(
                    "points",
                    format!(
                        "{},{} {},{} {},{}",
                        node.points[triangle.a as usize].xyz.x as f32 / 65536.0,
                        node.points[triangle.a as usize].xyz.y as f32 / 65536.0,
                        node.points[triangle.b as usize].xyz.x as f32 / 65536.0,
                        node.points[triangle.b as usize].xyz.y as f32 / 65536.0,
                        node.points[triangle.c as usize].xyz.x as f32 / 65536.0,
                        node.points[triangle.c as usize].xyz.y as f32 / 65536.0,
                    ),
                );

            document = document.add(poly);
        }
    }

    document = document
        .set("viewBox", (x.0, y.0, x.1 - x.0, y.1 - y.0))
        .set(
            "style",
            format!(
                "background-color: rgb({}, {}, {})",
                smx.ground_colour.r, smx.ground_colour.g, smx.ground_colour.b,
            ),
        );

    svg::save(args.output, &document).unwrap();
}
