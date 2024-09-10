#![feature(iterator_try_collect)]

use clap::Parser;
use lopdf::{dictionary, Document, Object, Result};
use std::ops::Add;

#[derive(Copy, Clone, Debug)]
struct Coordinate(f32, f32);

impl Add<Coordinate> for Coordinate {
    type Output = Coordinate;

    fn add(self, rhs: Coordinate) -> Coordinate {
        Coordinate(self.0 + rhs.0, self.1 + rhs.1)
    }
}

#[derive(Copy, Clone, Debug)]
struct BBox {
    lower_left: Coordinate,
    upper_right: Coordinate,
}

impl BBox {
    fn height(&self) -> f32 {
        self.upper_right.1 - self.lower_left.1
    }

    fn from_llur(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        let lower_left = Coordinate(x0, y0);
        let upper_right = Coordinate(x1, y1);
        Self {
            lower_left,
            upper_right,
        }
    }

    // Construct a BBox from (left, top, width, height)
    fn from_ltwh(left: f32, top: f32, width: f32, height: f32) -> Self {
        let lower_left = Coordinate(left, top - height);
        let upper_right = Coordinate(left + width, top);
        Self {
            lower_left,
            upper_right,
        }
    }

    fn as_vec(&self) -> Vec<Object> {
        vec![
            Object::Real(self.lower_left.0),
            Object::Real(self.lower_left.1),
            Object::Real(self.upper_right.0),
            Object::Real(self.upper_right.1),
        ]
    }

    fn as_quad_vec(&self) -> Vec<Object> {
        vec![
            Object::Real(self.lower_left.0),
            Object::Real(self.lower_left.1),
            Object::Real(self.upper_right.0),
            Object::Real(self.lower_left.1),
            Object::Real(self.upper_right.0),
            Object::Real(self.upper_right.1),
            Object::Real(self.lower_left.0),
            Object::Real(self.upper_right.1),
        ]
    }

    fn offset_within(self, other: BBox) -> BBox {
        let lower_left = Coordinate(
            self.lower_left.0 + other.lower_left.0,
            other.upper_right.1 - self.upper_right.1 - self.height(),
        );
        let upper_right = Coordinate(
            self.upper_right.0 + other.lower_left.0,
            other.upper_right.1 - self.upper_right.1,
        );
        BBox {
            lower_left,
            upper_right,
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(required = true)]
    path: String,
    #[arg()]
    out_path: Option<String>,
    #[arg(long, required = true)]
    page: u32,
    #[arg(long, required = true)]
    left: f32,
    #[arg(long, required = true)]
    top: f32,
    #[arg(long, required = true)]
    width: f32,
    #[arg(long, required = true)]
    height: f32,
}

fn mediabox(document: &Document, page_num: u32) -> Result<BBox> {
    let catalog = document.catalog()?;
    let pages = catalog.get_deref(b"Pages", &document)?;
    let mediabox = {
        if let Ok(mediabox) = pages.as_dict()?.get(b"MediaBox") {
            mediabox
        } else {
            let page_id = document.get_pages()[&page_num];
            let page = document.get_dictionary(page_id)?;
            page.get(b"MediaBox")?
        }
    };
    let mut mediabox_floats = mediabox.as_array()?.into_iter().map(|o| o.as_float());

    let out = BBox::from_llur(
        mediabox_floats.next().unwrap()?,
        mediabox_floats.next().unwrap()?,
        mediabox_floats.next().unwrap()?,
        mediabox_floats.next().unwrap()?,
    );

    Ok(out)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut document = Document::load(&cli.path)?;
    let doc_bbox = mediabox(&document, cli.page)?;
    let bbox = BBox::from_ltwh(cli.left, cli.top, cli.width, cli.height);
    let bbox = bbox.offset_within(doc_bbox);
    let page_id = document.get_pages()[&cli.page];
    let highlight_id = document.add_object(dictionary! {
        "Type" => "Annot",
        "Subtype" => "Highlight",
        "Rect" => bbox.as_vec(),
        "QuadPoints" => bbox.as_quad_vec(),
    });
    let page = document.get_dictionary_mut(page_id)?;
    if page.has(b"Annots") {
        let annots = page.get_mut(b"Annots")?.as_array_mut()?;
        annots.push(highlight_id.into());
    } else {
        page.set("Annots", vec![highlight_id.into()]);
    }
    document.save(cli.out_path.unwrap_or(cli.path))?;
    Ok(())
}
