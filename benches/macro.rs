use criterion::{black_box, criterion_group, criterion_main, Criterion};

/// Loops through the attributes once and pulls out the ones we ask it to. It
/// will check that the required ones are there. This could have been done with
/// attrs.find but that would be inefficient.
macro_rules! get_attrs {
    (
        for $attr:ident in $attrs:ident {
            $($branches:tt)*
        }
        $ret_expr:expr
    ) => {
        {
            $crate::let_attr_branches!($($branches)*);

            for attr in $attrs.iter() {
                let $attr = attr.value.clone();
                $crate::process_attr_branches!(attr; $($branches)*);
            }

            $crate::handle_attr_branches!($($branches)*);

            $ret_expr
        }
    };
}

macro_rules! let_attr_branches {
    () => {};

    (Some($attr_pat_opt:literal) => $opt_var:ident $(?)?= $opt_expr:expr $(, $($tail:tt)*)?) => {
        let mut $opt_var = None;
        $crate::let_attr_branches!($($($tail)*)?);
    };

    ($attr_pat_opt:literal => $opt_var:ident $(?)?= $opt_expr:expr $(, $($tail:tt)*)?) => {
        let mut $opt_var = None;
        $crate::let_attr_branches!($($($tail)*)?);
    };
}

pub(crate) use let_attr_branches;

macro_rules! process_attr_branches {
    ($attr:ident; ) => {};

    ($attr:ident; Some($attr_pat_opt:literal) => $opt_var:ident = $opt_expr:expr $(, $($tail:tt)*)?) => {
        if(&$attr.name.local_name == $attr_pat_opt) {
            $opt_var = Some($opt_expr);
        }
        else {
            $crate::process_attr_branches!($attr; $($($tail)*)?);
        }
    };

    ($attr:ident; Some($attr_pat_opt:literal) => $opt_var:ident ?= $opt_expr:expr $(, $($tail:tt)*)?) => {
        if(&$attr.name.local_name == $attr_pat_opt) {
            $opt_var = Some($opt_expr.map_err(|_|
                tiled::Error::MalformedAttributes(
                    concat!("Error parsing optional attribute '", $attr_pat_opt, "'").to_owned()
                )
            )?);
        }
        else {
            $crate::process_attr_branches!($attr; $($($tail)*)?);
        }
    };

    ($attr:ident; $attr_pat_opt:literal => $opt_var:ident = $opt_expr:expr $(, $($tail:tt)*)?) => {
        if(&$attr.name.local_name == $attr_pat_opt) {
            $opt_var = Some($opt_expr);
        }
        else {
            $crate::process_attr_branches!($attr; $($($tail)*)?);
        }
    };

    ($attr:ident; $attr_pat_opt:literal => $opt_var:ident ?= $opt_expr:expr $(, $($tail:tt)*)?) => {
        if(&$attr.name.local_name == $attr_pat_opt) {
            $opt_var = Some($opt_expr.map_err(|_|
                tiled::Error::MalformedAttributes(
                    concat!("Error parsing attribute '", $attr_pat_opt, "'").to_owned()
                )
            )?);
        }
        else {
            $crate::process_attr_branches!($attr; $($($tail)*)?);
        }
    }
}

pub(crate) use process_attr_branches;

macro_rules! handle_attr_branches {
    () => {};

    (Some($attr_pat_opt:literal) => $opt_var:ident $(?)?= $opt_expr:expr $(, $($tail:tt)*)?) => {
        $crate::handle_attr_branches!($($($tail)*)?);
    };

    ($attr_pat_opt:literal => $opt_var:ident $(?)?= $opt_expr:expr $(, $($tail:tt)*)?) => {
        let $opt_var = $opt_var
            .ok_or_else(||
                tiled::Error::MalformedAttributes(
                    concat!("Missing attribute: ", $attr_pat_opt).to_owned()
                )
            )?;

        $crate::handle_attr_branches!($($($tail)*)?);
    };
}

pub(crate) use handle_attr_branches;
use tiled::{Color, Orientation};
use xml::attribute::OwnedAttribute;

pub fn criterion_benchmark(c: &mut Criterion) {
    use xml::reader::EventReader;
    const TO_READ: &str = r#"<map version="1.4" tiledversion="1.4.0" orientation="orthogonal" renderorder="right-down" width="100" height="100" tilewidth="32" tileheight="32" infinite="0" nextlayerid="2" nextobjectid="1">"#;
    c.bench_function("new macro get_attrs", |b| {
        let mut reader = EventReader::new(std::io::Cursor::new(TO_READ));
        reader.next();
        match reader.next().unwrap() {
            xml::reader::XmlEvent::StartElement { attributes, .. } => b.iter(|| {
                process_attrs(&attributes[..]).unwrap();
            }),
            _ => panic!(),
        }
    });
}

fn process_attrs(attributes: &[OwnedAttribute]) -> Result<(), tiled::Error> {
    let (c, infinite, v, o, w, h, tw, th) = get_attrs!(
        for v in attributes {
            Some("backgroundcolor") => colour ?= v.parse::<Color>(),
            Some("infinite") => infinite = v == "1",
            "version" => version = v,
            "orientation" => orientation ?= v.parse::<Orientation>(),
            "width" => width ?= v.parse::<u32>(),
            "height" => height ?= v.parse::<u32>(),
            "tilewidth" => tile_width ?= v.parse::<u32>(),
            "tileheight" => tile_height ?= v.parse::<u32>(),
        }
        (colour, infinite, version, orientation, width, height, tile_width, tile_height)
    );

    black_box((c, infinite, v, o, w, h, tw, th));

    Ok(())
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
