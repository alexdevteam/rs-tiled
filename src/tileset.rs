use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use xml::attribute::OwnedAttribute;
use xml::reader::XmlEvent;
use xml::EventReader;

use crate::error::TiledError;
use crate::image::Image;
use crate::properties::Properties;
use crate::tile::{Gid, Tile};
use crate::util::{get_attrs, parse_tag};

/// A tileset, usually the tilesheet image.
#[derive(Debug, PartialEq, Clone)]
pub struct Tileset {
    /// The GID of the first tile stored.
    pub first_gid: Gid,
    pub name: String,
    pub tile_width: u32,
    pub tile_height: u32,
    pub spacing: u32,
    pub margin: u32,
    pub tilecount: u32,
    /// The Tiled spec says that a tileset can have mutliple images so a `Vec`
    /// is used. Usually you will only use one.
    pub images: Vec<Image>,
    pub tiles: Vec<Tile>,
    pub properties: Properties,
    /// Where this tileset was loaded from.
    /// If embedded in a map file, this will be the map path.
    /// If fully embedded (loaded with path = `None`), this will return `None`.
    pub source: Option<PathBuf>,
}

impl Tileset {
    /// Parse a buffer hopefully containing the contents of a Tiled tileset.
    ///
    /// External tilesets do not have a firstgid attribute.  That lives in the
    /// map. You must pass in `first_gid`.  If you do not need to use gids for anything,
    /// passing in 1 will work fine.
    ///
    /// The path may be skipped if the tileset is fully embedded (Doesn't refer to external files).
    pub fn parse_reader<R: Read>(
        reader: R,
        first_gid: Gid,
        path: Option<&Path>,
    ) -> Result<Self, TiledError> {
        Tileset::new_external(reader, first_gid, path)
    }

    pub fn contains_tile(&self, gid: Gid) -> bool {
        self.first_gid <= gid && gid.0 < self.first_gid.0 + self.tilecount
    }

    pub(crate) fn parse_xml<R: Read>(
        parser: &mut EventReader<R>,
        attrs: Vec<OwnedAttribute>,
        map_path: Option<&Path>,
    ) -> Result<Tileset, TiledError> {
        Tileset::parse_xml_embedded(parser, &attrs, map_path)
            .or_else(|_| Tileset::parse_xml_reference(&attrs, map_path))
    }

    fn parse_xml_embedded<R: Read>(
        parser: &mut EventReader<R>,
        attrs: &Vec<OwnedAttribute>,
        map_path: Option<&Path>,
    ) -> Result<Tileset, TiledError> {
        let ((spacing, margin), (tilecount, first_gid, name, width, height)) = get_attrs!(
           attrs,
           optionals: [
                ("spacing", spacing, |v:String| v.parse().ok()),
                ("margin", margin, |v:String| v.parse().ok()),
            ],
           required: [
                ("tilecount", tilecount, |v:String| v.parse().ok()),
                ("firstgid", first_gid, |v:String| v.parse().ok().and_then(|i| Some(Gid(i)))),
                ("name", name, |v| Some(v)),
                ("tilewidth", width, |v:String| v.parse().ok()),
                ("tileheight", height, |v:String| v.parse().ok()),
            ],
            TiledError::MalformedAttributes("tileset must have a firstgid, name tile width and height with correct types".to_string())
        );

        let mut images = Vec::new();
        let mut tiles = Vec::new();
        let mut properties = Properties::default();
        parse_tag!(parser, "tileset", {
            "image" => |attrs| {
                images.push(Image::new(parser, attrs)?);
                Ok(())
            },
            "properties" => |_| {
                properties = Properties::parse_xml(parser)?;
                Ok(())
            },
            "tile" => |attrs| {
                tiles.push(Tile::new(parser, attrs)?);
                Ok(())
            },
        });

        Ok(Tileset {
            tile_width: width,
            tile_height: height,
            spacing: spacing.unwrap_or(0),
            margin: margin.unwrap_or(0),
            first_gid,
            name,
            tilecount,
            images,
            tiles,
            properties,
            source: map_path.and_then(|p| Some(p.to_owned())),
        })
    }

    fn parse_xml_reference(
        attrs: &Vec<OwnedAttribute>,
        map_path: Option<&Path>,
    ) -> Result<Tileset, TiledError> {
        let ((), (first_gid, source)) = get_attrs!(
            attrs,
            optionals: [],
            required: [
                ("firstgid", first_gid, |v:String| v.parse().ok().and_then(|i| Some(Gid(i)))),
                ("source", name, |v| Some(v)),
            ],
            TiledError::MalformedAttributes("tileset must have a firstgid, name tile width and height with correct types".to_string())
        );

        let tileset_path = map_path.ok_or(TiledError::Other("Maps with external tilesets must know their file location.  See parse_with_path(Path).".to_string()))?.with_file_name(source);
        let file = File::open(&tileset_path).map_err(|_| {
            TiledError::Other(format!(
                "External tileset file not found: {:?}",
                tileset_path
            ))
        })?;
        Tileset::new_external(file, first_gid, Some(&tileset_path))
    }

    pub(crate) fn new_external<R: Read>(
        file: R,
        first_gid: Gid,
        source: Option<&Path>,
    ) -> Result<Tileset, TiledError> {
        let mut tileset_parser = EventReader::new(file);
        loop {
            match tileset_parser
                .next()
                .map_err(TiledError::XmlDecodingError)?
            {
                XmlEvent::StartElement {
                    name, attributes, ..
                } => {
                    if name.local_name == "tileset" {
                        return Tileset::parse_external_tileset(
                            first_gid,
                            &mut tileset_parser,
                            &attributes,
                            source,
                        );
                    }
                }
                XmlEvent::EndDocument => {
                    return Err(TiledError::PrematureEnd(
                        "Tileset Document ended before map was parsed".to_string(),
                    ))
                }
                _ => {}
            }
        }
    }

    fn parse_external_tileset<R: Read>(
        first_gid: Gid,
        parser: &mut EventReader<R>,
        attrs: &Vec<OwnedAttribute>,
        source: Option<&Path>,
    ) -> Result<Tileset, TiledError> {
        let ((spacing, margin), (tilecount, name, width, height)) = get_attrs!(
            attrs,
            optionals: [
                ("spacing", spacing, |v:String| v.parse().ok()),
                ("margin", margin, |v:String| v.parse().ok()),
            ],
            required: [
                ("tilecount", tilecount, |v:String| v.parse().ok()),
                ("name", name, |v| Some(v)),
                ("tilewidth", width, |v:String| v.parse().ok()),
                ("tileheight", height, |v:String| v.parse().ok()),
            ],
            TiledError::MalformedAttributes("tileset must have a firstgid, name tile width and height with correct types".to_string())
        );

        let mut images = Vec::new();
        let mut tiles = Vec::new();
        let mut properties = Properties::default();
        parse_tag!(parser, "tileset", {
            "image" => |attrs| {
                images.push(Image::new(parser, attrs)?);
                Ok(())
            },
            "tile" => |attrs| {
                tiles.push(Tile::new(parser, attrs)?);
                Ok(())
            },
            "properties" => |_| {
                properties = Properties::parse_xml(parser)?;
                Ok(())
            },
        });

        Ok(Tileset {
            first_gid,
            name,
            tile_width: width,
            tile_height: height,
            spacing: spacing.unwrap_or(0),
            margin: margin.unwrap_or(0),
            tilecount,
            images,
            tiles,
            properties,
            source: source.and_then(|p| Some(p.to_owned())),
        })
    }

    pub fn get_tile_by_gid(&self, gid: Gid) -> Option<&Tile> {
        let id = gid.0 - self.first_gid.0;
        // FIXME: This won't return tiles with no special characteristics (tiled crate/format only keeps track
        // of ones with metadata attached)
        match self
            .tiles
            .binary_search_by_key(&id, |t| t.id)
        {
            Ok(index) => Some(&self.tiles[index]),
            Err(_) => todo!(),
        }
    }
}
