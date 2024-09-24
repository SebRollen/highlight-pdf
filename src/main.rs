#![feature(iterator_try_collect)]

mod bbox;
mod cli;

use bbox::BBox;
use clap::Parser;
use cli::Cli;
use lopdf::{dictionary, Document, Object, Result};

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
    let bbox = cli.get_bbox();
    let bbox = bbox.offset_within(doc_bbox);
    let page_id = document.get_pages()[&cli.page];
    let highlight_id = document.add_object(dictionary! {
        "Type" => "Annot",
        "Subtype" => "Highlight",
        "Rect" => bbox.as_vec(),
        "QuadPoints" => bbox.as_quad_vec(),
    });
    let page = document.get_dictionary(page_id)?;
    if page.has(b"Annots") {
        let array: &mut Vec<Object> = match page.get(b"Annots")? {
            Object::Reference(id) => document.get_object_mut(*id)?.as_array_mut()?,
            Object::Array(_) => {
                // We have to reborrow everything as mutable here
                document
                    .get_dictionary_mut(page_id)?
                    .get_mut(b"Annots")?
                    .as_array_mut()?
            }
            _ => unreachable!(),
        };
        array.push(highlight_id.into());
    } else {
        let page = document.get_dictionary_mut(page_id)?;
        page.set("Annots", vec![highlight_id.into()]);
    }
    document.save(cli.out_path.unwrap_or(cli.path))?;
    Ok(())
}
