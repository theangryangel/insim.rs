use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;
use insim_pth::Pth;

use crate::SCALE;

#[derive(Debug, Args)]
pub(crate) struct SimpleArgs {
    #[clap(short, long)]
    pth: PathBuf,
}

impl SimpleArgs {
    pub(crate) fn run(&self, viewbox_padding: f32) -> Result<svg::Document> {
        let mut document = svg::Document::new();
        let mut viewbox_x: (f32, f32) = (0.0, 0.0);
        let mut viewbox_y: (f32, f32) = (0.0, 0.0);

        let p = Pth::from_path(&self.pth).context(format!("Failed to read {:?}", &self.pth))?;

        let first = p.iter_nodes().next().unwrap().get_center(SCALE.into());

        let mut data = svg::node::element::path::Data::new().move_to((first.x, first.y));

        for node in p.iter_nodes() {
            let point = node.get_center(SCALE.into());

            viewbox_x.0 = viewbox_x.0.min(point.x);
            viewbox_x.1 = viewbox_x.1.max(point.x);

            viewbox_y.0 = viewbox_y.0.min(point.y);
            viewbox_y.1 = viewbox_y.1.max(point.y);

            data = data.line_to((point.x, point.y));
        }

        data = data.close();

        let background = svg::node::element::Path::new()
            .set("fill", "none")
            .set("stroke", "#111111")
            .set("stroke-width", 20)
            .set("stroke-linejoin", "round")
            .set("d", data.clone());

        let line = svg::node::element::Path::new()
            .set("fill", "none")
            .set("stroke", "white")
            .set("stroke-width", 10)
            .set("stroke-linejoin", "round")
            .set("d", data.clone());

        document = document.add(background);
        document = document.add(line);

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
