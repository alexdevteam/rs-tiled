use std::path::Path;
use std::{fs::File, path::PathBuf};
use tiled::{
    error::TiledError, layers::LayerData, map::Map, properties::PropertyValue, tile::Gid,
    tileset::Tileset,
};

fn parse_map_without_source(p: &Path) -> Result<Map, TiledError> {
    let file = File::open(p).unwrap();
    return Map::parse_reader(file, None);
}

fn assert_eq_map_without_source(r: &Map, e: &Map) {
    assert_eq!(r.version, e.version);
    assert_eq!(r.orientation, e.orientation);
    assert_eq!(r.width, e.width);
    assert_eq!(r.height, e.height);
    assert_eq!(r.tile_width, e.tile_width);
    assert_eq!(r.tile_height, e.tile_height);
    assert_eq!(r.layers, e.layers);
    assert_eq!(r.image_layers, e.image_layers);
    assert_eq!(r.object_groups, e.object_groups);
    assert_eq!(r.properties, e.properties);
    assert_eq!(r.background_color, e.background_color);
    assert_eq!(r.infinite, e.infinite);
    assert_eq!(r.tilesets.len(), e.tilesets.len());
    r.tilesets
        .iter()
        .zip(e.tilesets.iter())
        .for_each(|(t, t2)| assert_eq_tileset_without_source(t, t2));
}

fn assert_eq_tileset_without_source(t2: &Tileset, t: &Tileset) {
    assert_eq!(t2.first_gid, t.first_gid);
    assert_eq!(t2.name, t.name);
    assert_eq!(t2.tile_width, t.tile_width);
    assert_eq!(t2.tile_height, t.tile_height);
    assert_eq!(t2.spacing, t.spacing);
    assert_eq!(t2.margin, t.margin);
    assert_eq!(t2.tilecount, t.tilecount);
    assert_eq!(t2.images, t.images);
    assert_eq!(t2.tiles, t.tiles);
    assert_eq!(t2.properties, t.properties);
}

#[test]
fn test_gzip_and_zlib_encoded_and_raw_are_the_same() {
    let z = parse_map_without_source(&Path::new("assets/tiled_base64_zlib.tmx")).unwrap();
    let g = parse_map_without_source(&Path::new("assets/tiled_base64_gzip.tmx")).unwrap();
    let r = parse_map_without_source(&Path::new("assets/tiled_base64.tmx")).unwrap();
    let c = parse_map_without_source(&Path::new("assets/tiled_csv.tmx")).unwrap();
    assert_eq!(z, g);
    assert_eq!(z, r);
    assert_eq!(z, c);

    if let LayerData::Finite(tiles) = &c.layers[0].tiles {
        assert_eq!(tiles.len(), 100);
        assert_eq!(tiles[0].len(), 100);
        assert_eq!(tiles[99].len(), 100);
        assert_eq!(tiles[0][0].gid, Gid(35));
        assert_eq!(tiles[1][0].gid, Gid(17));
        assert_eq!(tiles[2][0].gid, Gid(0));
        assert_eq!(tiles[2][1].gid, Gid(17));
        assert!(tiles[99].iter().map(|t| t.gid).all(|g| g == Gid::EMPTY));
    } else {
        assert!(false, "It is wrongly recognised as an infinite map");
    }
}

#[test]
fn test_external_tileset() {
    let r = parse_map_without_source(&Path::new("assets/tiled_base64.tmx")).unwrap();
    let e = Map::parse_file(&Path::new("assets/tiled_base64_external.tmx")).unwrap();
    // Compare everything BUT source
    assert_eq_map_without_source(&r, &e);
}

#[test]
fn test_just_tileset() {
    let r = Map::parse_file(&Path::new("assets/tiled_base64.tmx")).unwrap();
    let path = Path::new("assets/tilesheet.tsx");
    let t = Tileset::parse_reader(File::open(path).unwrap(), Gid(1), Some(path)).unwrap();
    let t2 = &r.tilesets[0];
    // Compare everything BUT source
    assert_eq_tileset_without_source(&t, &t2);
}

#[test]
fn test_infinite_tileset() {
    let r = Map::parse_file(&Path::new("assets/tiled_base64_zlib_infinite.tmx")).unwrap();

    if let LayerData::Infinite(chunks) = &r.layers[0].tiles {
        assert_eq!(chunks.len(), 4);

        assert_eq!(chunks[&(0, 0)].width, 32);
        assert_eq!(chunks[&(0, 0)].height, 32);
        assert_eq!(chunks[&(-32, 0)].width, 32);
        assert_eq!(chunks[&(0, 32)].height, 32);
        assert_eq!(chunks[&(-32, 32)].height, 32);
    } else {
        assert!(false, "It is wrongly recognised as a finite map");
    }
}

#[test]
fn test_sources() {
    let external = Map::parse_file(Path::new("assets/tiled_base64_external.tmx")).unwrap();
    assert_eq!(
        external.source,
        Some(Path::new("assets/tiled_base64_external.tmx").to_owned())
    );
    assert_eq!(
        external.tilesets[0].source,
        Some(Path::new("assets/tilesheet.tsx").to_owned())
    );

    let embedded = parse_map_without_source(&Path::new("assets/tiled_base64.tmx")).unwrap();
    assert_eq!(embedded.source, None);
    assert_eq!(embedded.tilesets[0].source, None);
}

#[test]
fn test_external_tileset_from_embedded_map() {
    match parse_map_without_source(&Path::new("assets/tiled_base64_external.tmx")).unwrap_err() {
        TiledError::Other(err) if err == "Maps with external tilesets must know their file location.  See parse_with_path(Path)." => (),
        _ => panic!()
    }
}

#[test]
fn test_image_layers() {
    let r = parse_map_without_source(&Path::new("assets/tiled_image_layers.tmx")).unwrap();
    assert_eq!(r.image_layers.len(), 2);
    {
        let first = &r.image_layers[0];
        assert_eq!(first.name, "Image Layer 1");
        assert!(
            first.image.is_none(),
            "{}'s image should be None",
            first.name
        );
    }
    {
        let second = &r.image_layers[1];
        assert_eq!(second.name, "Image Layer 2");
        let image = second
            .image
            .as_ref()
            .expect(&format!("{}'s image shouldn't be None", second.name));
        assert_eq!(image.source, "tilesheet.png");
        assert_eq!(image.width, 448);
        assert_eq!(image.height, 192);
    }
}

#[test]
fn test_tile_property() {
    let r = parse_map_without_source(&Path::new("assets/tiled_base64.tmx")).unwrap();
    let prop_value: String = if let Some(&PropertyValue::StringValue(ref v)) =
        r.tilesets[0].tiles[0].properties.0.get("a tile property")
    {
        v.clone()
    } else {
        String::new()
    };
    assert_eq!("123", prop_value);
}

#[test]
fn test_object_group_property() {
    let r = parse_map_without_source(&Path::new("assets/tiled_object_groups.tmx")).unwrap();
    let prop_value: bool = if let Some(&PropertyValue::BoolValue(ref v)) = r.object_groups[0]
        .properties
        .0
        .get("an object group property")
    {
        *v
    } else {
        false
    };
    assert!(prop_value);
}
#[test]
fn test_tileset_property() {
    let r = parse_map_without_source(&Path::new("assets/tiled_base64.tmx")).unwrap();
    let prop_value: String = if let Some(&PropertyValue::StringValue(ref v)) =
        r.tilesets[0].properties.0.get("tileset property")
    {
        v.clone()
    } else {
        String::new()
    };
    assert_eq!("tsp", prop_value);
}

#[test]
fn test_flipped_gid() {
    let r = Map::parse_file(&Path::new("assets/tiled_flipped.tmx")).unwrap();

    if let LayerData::Finite(tiles) = &r.layers[0].tiles {
        let t1 = tiles[0][0];
        let t2 = tiles[0][1];
        let t3 = tiles[1][0];
        let t4 = tiles[1][1];
        assert_eq!(t1.gid, t2.gid);
        assert_eq!(t2.gid, t3.gid);
        assert_eq!(t3.gid, t4.gid);
        assert!(t1.flip_d);
        assert!(t1.flip_h);
        assert!(t1.flip_v);
        assert!(!t2.flip_d);
        assert!(!t2.flip_h);
        assert!(t2.flip_v);
        assert!(!t3.flip_d);
        assert!(t3.flip_h);
        assert!(!t3.flip_v);
        assert!(t4.flip_d);
        assert!(!t4.flip_h);
        assert!(!t4.flip_v);
    } else {
        assert!(false, "It is wrongly recognised as an infinite map");
    }
}

#[test]
fn test_ldk_export() {
    let r = Map::parse_file(&Path::new("assets/ldk_tiled_export.tmx")).unwrap();
    if let LayerData::Finite(tiles) = &r.layers[0].tiles {
        assert_eq!(tiles.len(), 8);
        assert_eq!(tiles[0].len(), 8);
        assert_eq!(tiles[0][0].gid, Gid::EMPTY);
        assert_eq!(tiles[1][0].gid, Gid(1));
    } else {
        assert!(false, "It is wrongly recognised as an infinite map");
    }
}
