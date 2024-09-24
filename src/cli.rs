use crate::BBox;
use clap::{Args, Parser};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// The PDF to add a highlight to.
    #[arg(required = true)]
    pub path: String,
    /// If set, a new PDF will be generated with the highlight at this path.
    /// If unset, the PDF at `path` will be updated in place.
    #[arg()]
    pub out_path: Option<String>,
    /// The page on which to render the highlight bounding box.
    #[arg(long, required = true)]
    pub page: u32,
    /// The left of the highlight bounding box, in pixels.
    #[arg(long, required = true)]
    pub left: f32,
    /// The top of the highlight bounding box, in pixels.
    #[arg(long, required = true)]
    pub top: f32,
    #[command(flatten)]
    x: X,
    #[command(flatten)]
    y: Y,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
struct X {
    /// The width of the highlight bounding box, in pixels.
    #[arg(long)]
    pub width: Option<f32>,
    /// The right of the highlight bounding box, in pixels.
    #[arg(long)]
    pub right: Option<f32>,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
struct Y {
    /// The height of the highlight bounding box, in pixels.
    #[arg(long)]
    pub height: Option<f32>,
    /// The bottom of the highlight bounding box, in pixels.
    #[arg(long)]
    pub bottom: Option<f32>,
}

impl Cli {
    pub fn get_bbox(&self) -> BBox {
        let width = self
            .x
            .width
            .unwrap_or_else(|| self.x.right.expect("guaranteed to be set") - self.left);
        let height = self
            .y
            .height
            .unwrap_or_else(|| self.top - self.y.bottom.expect("guaranteed to be set"));
        BBox::from_ltwh(self.left, self.top, width, height)
    }
}
