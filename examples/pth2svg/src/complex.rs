use std::path;

use anyhow::{Context, Result};
use clap::Args;
use insim_pth::{Pth, node::Node};

use crate::SCALE;

#[derive(Args, Debug)]
pub(crate) struct ComplexArgs {
    #[clap(short, long)]
    pth: Vec<path::PathBuf>,

    #[clap(long)]
    racing_line: Option<path::PathBuf>,

    #[clap(long, default_value_t = String::from("#3D9970"))]
    background_colour: String,

    #[clap(long, default_value_t = String::from("#111111"))]
    track_colour: String,

    #[clap(long, default_value_t = String::from("#FF4136"))]
    racing_line_colour: String,
}

impl ComplexArgs {
    pub(crate) fn run(self, viewbox_padding: f32) -> Result<svg::Document> {
        let mut document = svg::Document::new();

        let mut all_pth_nodes = Vec::new();

        let mut viewbox_x: (f32, f32) = (0.0, 0.0);
        let mut viewbox_y: (f32, f32) = (0.0, 0.0);

        for i in self.pth.iter() {
            let p = Pth::from_path(i)?;

            // wrap around the nodes to avoid missing "notches" in the track drawing
            let mut nodes: Vec<Node> = p.iter_nodes().cloned().collect();
            nodes.insert(0, *nodes.last().unwrap());

            all_pth_nodes.push(nodes);
        }

        // draw all the track limits first, a single polygon per PTH
        // to avoid the "gaps" issue
        for nodes in all_pth_nodes.iter() {
            let mut fwd = Vec::with_capacity(nodes.len() * 2);
            let mut bck = Vec::with_capacity(nodes.len());

            for node in nodes.iter() {
                let limits = node.get_outer_limit(SCALE.into());

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
                .set("style", format!("fill: {}", self.background_colour))
                .set(
                    "points",
                    fwd.iter()
                        .map(|i| format!("{},{}", i.0, i.1))
                        .collect::<Vec<String>>()
                        .join(" "),
                );

            document = document.add(poly);
        }

        // draw all the roads next
        for nodes in all_pth_nodes.iter() {
            let mut fwd = Vec::with_capacity(nodes.len() * 2);
            let mut bck = Vec::with_capacity(nodes.len());

            for node in nodes.iter() {
                let limits = node.get_road_limit(SCALE.into());

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
                .set("style", format!("fill: {}", self.track_colour))
                .set("points", points);

            document = document.add(poly);
        }

        if let Some(i) = self.racing_line {
            let p = Pth::from_path(&i).context(format!("Failed to read {:?}", &i))?;

            let mut data = svg::node::element::path::Data::new().move_to((
                p.iter_nodes().next().unwrap().center.x as f32,
                p.iter_nodes().next().unwrap().center.y as f32,
            ));

            for node in p.iter_nodes() {
                data = data.line_to((node.center.x as f32, node.center.y as f32));
            }

            data = data.close();

            let path = svg::node::element::Path::new()
                .set("fill", "none")
                .set("stroke", self.racing_line_colour)
                .set("stroke-width", 2)
                .set("d", data);

            document = document.add(path);
        }

        document = document.set(
            "viewBox",
            (
                viewbox_x.0 - viewbox_padding,
                viewbox_y.0 - viewbox_padding,
                (viewbox_x.1 + viewbox_padding) - (viewbox_x.0 - viewbox_padding),
                (viewbox_y.1 + viewbox_padding) - (viewbox_y.0 - viewbox_padding),
            ),
        );

        Ok(document)
    }
}
