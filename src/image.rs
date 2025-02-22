use std::io::Read;

use xml::{attribute::OwnedAttribute, EventReader};

use crate::{error::TiledError, properties::Color, util::*};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Image {
    /// The filepath of the image
    pub source: String,
    pub width: i32,
    pub height: i32,
    pub transparent_color: Option<Color>,
}

impl Image {
    pub(crate) fn new<R: Read>(
        parser: &mut EventReader<R>,
        attrs: Vec<OwnedAttribute>,
    ) -> Result<Image, TiledError> {
        let (c, (s, w, h)) = get_attrs!(
            attrs,
            optionals: [
                ("trans", trans, |v:String| v.parse().ok()),
            ],
            required: [
                ("source", source, |v| Some(v)),
                ("width", width, |v:String| v.parse().ok()),
                ("height", height, |v:String| v.parse().ok()),
            ],
            TiledError::MalformedAttributes("image must have a source, width and height with correct types".to_string())
        );

        parse_tag!(parser, "image", { "" => |_| Ok(()) });
        Ok(Image {
            source: s,
            width: w,
            height: h,
            transparent_color: c,
        })
    }
}
