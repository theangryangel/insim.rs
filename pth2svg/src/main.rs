use clap::Parser;
use geojson::{
    feature::Id, Feature, FeatureCollection, GeoJson, Geometry, JsonObject, JsonValue, Value,
};
use insim::file::pth::Pth;
use miette::{IntoDiagnostic, Result, WrapErr};
use std::path;

const DEFAULT_SCALE: f32 = 65536.0;

/// pth2svg converts one or more PTH files to a simplified SVG image
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    pth: Vec<path::PathBuf>,

    #[clap(short, long)]
    output: path::PathBuf,

    #[clap(long)]
    racing_line: Option<path::PathBuf>,

    #[clap(long, default_value_t = 10.0)]
    viewbox_padding: f32,

    #[clap(long, default_value_t = String::from("#3D9970"))]
    background_colour: String,

    #[clap(long, default_value_t = String::from("#111111"))]
    track_colour: String,

    #[clap(long, default_value_t = String::from("#FF4136"))]
    racing_line_colour: String,

    #[clap(long)]
    scale: Option<f32>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.output.exists() {
        let err = Err(std::io::Error::from(std::io::ErrorKind::AlreadyExists));
        err.into_diagnostic()
            .wrap_err(format!("Output path already exists {:?}", &args.output))?;
    }

    let mut collection = FeatureCollection {
        bbox: None,
        features: vec![],
        foreign_members: None,
    };

    let mut document = svg::Document::new();

    let mut all_pth_nodes = Vec::new();

    let mut viewbox_x: (f32, f32) = (0.0, 0.0);
    let mut viewbox_y: (f32, f32) = (0.0, 0.0);

    for i in args.pth.iter() {
        let p = Pth::from_file(i).into_diagnostic()?;

        // wrap around the nodes to avoid missing "notches" in the track drawing
        let mut nodes = p.nodes.clone();
        nodes.insert(0, *nodes.last().unwrap());

        all_pth_nodes.push(nodes);
    }

    // draw all the track limits first, a single polygon per PTH
    // to avoid the "gaps" issue
    for nodes in all_pth_nodes.iter() {
        let mut fwd = Vec::with_capacity(nodes.len() * 2);
        let mut bck = Vec::with_capacity(nodes.len());

        for node in nodes.iter() {
            let limits = node.get_outer_limit(Some(args.scale.unwrap_or(DEFAULT_SCALE)));

            viewbox_x.0 = viewbox_x.0.min(limits.0.x);
            viewbox_x.0 = viewbox_x.0.min(limits.1.x);

            viewbox_x.1 = viewbox_x.1.max(limits.0.x);
            viewbox_x.1 = viewbox_x.1.max(limits.1.x);

            viewbox_y.0 = viewbox_y.0.min(limits.0.y);
            viewbox_y.0 = viewbox_y.0.min(limits.1.y);

            viewbox_y.1 = viewbox_y.1.max(limits.0.y);
            viewbox_y.1 = viewbox_y.1.max(limits.1.y);

            fwd.push((limits.0.x, limits.0.y));
            bck.push((limits.1.x, limits.1.y));
        }

        fwd.extend(bck.iter().rev());

        let poly = svg::node::element::Polygon::new()
            .set("style", format!("fill: {}", args.background_colour))
            .set(
                "points",
                fwd.iter()
                    .map(|i| format!("{},{}", i.0, i.1))
                    .collect::<Vec<String>>()
                    .join(" "),
            );

        document = document.add(poly);

        let geometry = Geometry::new(Value::Polygon(vec![fwd
            .iter()
            .map(|i| vec![i.1 as f64, i.0 as f64])
            .collect::<Vec<Vec<f64>>>()]));

        let geojson = Feature {
            bbox: None,
            geometry: Some(geometry),
            id: Some(Id::String("limit".into())),
            // See the next section about Feature properties
            properties: None,
            foreign_members: None,
        };

        collection.features.push(geojson);
    }

    // draw all the roads next
    for nodes in all_pth_nodes.iter() {
        let mut fwd = Vec::with_capacity(nodes.len() * 2);
        let mut bck = Vec::with_capacity(nodes.len());

        for node in nodes.iter() {
            let limits = node.get_road_limit(Some(args.scale.unwrap_or(DEFAULT_SCALE)));

            fwd.push((limits.0.x, limits.0.y));
            bck.push((limits.1.x, limits.1.y));
        }

        fwd.extend(bck.iter().rev());

        let points = fwd
            .iter()
            .map(|i| format!("{},{}", i.0, i.1))
            .collect::<Vec<String>>()
            .join(" ");

        let poly = svg::node::element::Polygon::new()
            .set("style", format!("fill: {}", args.track_colour))
            .set("points", points);

        let geometry = Geometry::new(Value::Polygon(vec![fwd
            .iter()
            .map(|i| vec![i.1 as f64, i.0 as f64])
            .collect::<Vec<Vec<f64>>>()]));

        let geojson = Feature {
            bbox: None,
            geometry: Some(geometry),
            id: Some(Id::String("track".into())),
            // See the next section about Feature properties
            properties: None,
            foreign_members: None,
        };

        collection.features.push(geojson);

        document = document.add(poly);
    }

    if let Some(i) = args.racing_line {
        let p = Pth::from_file(&i)
            .into_diagnostic()
            .wrap_err(format!("Failed to read {:?}", &i))?;

        let mut data = svg::node::element::path::Data::new().move_to((
            p.nodes.first().unwrap().center.x as f32 / args.scale.unwrap_or(DEFAULT_SCALE),
            p.nodes.first().unwrap().center.y as f32 / args.scale.unwrap_or(DEFAULT_SCALE),
        ));

        for node in p.nodes.iter() {
            data = data.line_to((
                node.center.x as f32 / args.scale.unwrap_or(DEFAULT_SCALE),
                node.center.y as f32 / args.scale.unwrap_or(DEFAULT_SCALE),
            ));
        }

        data = data.close();

        let path = svg::node::element::Path::new()
            .set("fill", "none")
            .set("stroke", args.racing_line_colour)
            .set("stroke-width", 2)
            .set("d", data);

        document = document.add(path);
    }

    document = document.set(
        "viewBox",
        (
            viewbox_x.0 - args.viewbox_padding,
            viewbox_y.0 - args.viewbox_padding,
            (viewbox_x.1 + args.viewbox_padding) - (viewbox_x.0 - args.viewbox_padding),
            (viewbox_y.1 + args.viewbox_padding) - (viewbox_y.0 - args.viewbox_padding),
        ),
    );

    svg::save(&args.output, &document)
        .into_diagnostic()
        .wrap_err(format!("Could not save output SVG to '{:?}'", &args.output))?;

    let mut json = args.output.clone();
    json.set_extension("json");

    std::fs::write(json, GeoJson::from(collection).to_string()).into_diagnostic()?;

    Ok(())
}
