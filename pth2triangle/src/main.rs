use clap::Parser;
use insim_core::point::Point;
use insim_pth::Pth;
use itertools::Itertools;
use miette::{IntoDiagnostic, Result};
use serde::Serialize;
use std::path;

const DEFAULT_SCALE: f32 = 65536.0;

/// pth2triangle converts one or more PTH files to a simplified list of triangles
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    pth: Vec<path::PathBuf>,

    #[clap(short, long)]
    output: path::PathBuf,
}

#[derive(Default, Serialize)]
struct Output {
    name: String,
    collection: Vec<Points>,
}

#[derive(Default, Serialize)]
struct Points {
    source: String,
    triangles: Vec<(SimplePoint, SimplePoint, SimplePoint)>,
}

type SimplePoint = (f32, f32, f32);

fn main() -> Result<()> {
    let args = Args::parse();

    if args.output.exists() {
        let err = Err(std::io::Error::from(std::io::ErrorKind::AlreadyExists));
        err.into_diagnostic()?;
    }

    let mut output = Output::default();

    for i in args.pth {
        let p = Pth::from_file(&i).into_diagnostic()?;

        // wrap around the nodes to avoid missing "notches" in the track drawing
        let mut nodes = p.nodes.clone();

        let mut iter = p.nodes.iter().rev();

        while nodes.len() % 3 != 0 {
            nodes.insert(0, *iter.next().unwrap());
        }

        let points = nodes
            .iter()
            .flat_map(|n| {
                let (left, right) = n.get_road_limit(Some(DEFAULT_SCALE));

                [(left.x, left.y, left.z), (right.x, right.y, right.z)].to_vec()
            })
            .collect::<Vec<SimplePoint>>()
            .iter()
            .circular_tuple_windows::<(&SimplePoint, &SimplePoint, &SimplePoint)>()
            .map(|(one, two, three)| (one.clone(), two.clone(), three.clone()))
            .collect::<Vec<(SimplePoint, SimplePoint, SimplePoint)>>();

        output.collection.push(Points {
            source: i.file_name().unwrap().to_string_lossy().into(),
            triangles: points.to_vec(),
        });
    }

    std::fs::write(
        &args.output,
        serde_json::to_string(&output).into_diagnostic()?,
    )
    .into_diagnostic()?;

    Ok(())
}
