use std::{io::Read, time::Duration};

use xml::{attribute::OwnedAttribute, EventReader};

use crate::{
    error::TiledError,
    util::{get_attrs, parse_tag},
};

/// Describes a frame of a tile animation.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Frame {
    /// The local tile to switch to this frame.
    ///
    /// This is NOT a global ID but rather a local tileset ID that starts at 0.
    pub tile_id: u32,
    /// The duration that this frame should be displayed before switching to the next one.
    pub duration: Duration,
}

impl Frame {
    pub(crate) fn new(attrs: Vec<OwnedAttribute>) -> Result<Frame, TiledError> {
        let ((), (tile_id, duration)) = get_attrs!(
            attrs,
            optionals: [],
            required: [
                ("tileid", tile_id, |v:String| v.parse().ok()),
                ("duration", duration,
                |v:String| v.parse::<u32>()
                            .ok()
                            .and_then(|ms| Some(Duration::from_millis(ms as u64)))
                ),
            ],
            TiledError::MalformedAttributes("A frame must have tileid and duration".to_string())
        );
        Ok(Frame {
            tile_id: tile_id,
            duration: duration,
        })
    }
}

/// Describes a tile animation.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Animation {
    pub frames: Vec<Frame>,
}

impl Animation {
    pub fn parse_xml<R: Read>(parser: &mut EventReader<R>) -> Result<Self, TiledError> {
        let mut frames = Vec::new();
        parse_tag!(parser, "animation", {
            "frame" => |attrs| {
                frames.push(Frame::new(attrs)?);
                Ok(())
            },
        });
        Ok(Self { frames })
    }
}
