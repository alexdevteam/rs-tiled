use std::io::Cursor;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tiled::{Map, ResourceCache};

struct NoopResourceCache;

impl ResourceCache for NoopResourceCache {
    fn get_tileset(
        &self,
        _path: impl AsRef<tiled::ResourcePath>,
    ) -> Option<std::sync::Arc<tiled::Tileset>> {
        unimplemented!()
    }

    fn get_or_try_insert_tileset_with<F, E>(
        &mut self,
        _path: tiled::ResourcePathBuf,
        _f: F,
    ) -> Result<std::sync::Arc<tiled::Tileset>, E>
    where
        F: FnOnce() -> Result<tiled::Tileset, E>,
    {
        unimplemented!()
    }
}

pub fn map_loading(c: &mut Criterion) {
    c.bench_function("parse tiled_csv.tmx", |b| {
        b.iter(|| {
            Map::parse_reader(
                black_box(Cursor::new(include_str!("../assets/tiled_csv.tmx"))),
                "assets/tiled_csv.tmx",
                &mut NoopResourceCache,
            )
        })
    });
    c.bench_function("parse tiled_base64.tmx", |b| {
        b.iter(|| {
            Map::parse_reader(
                black_box(Cursor::new(include_str!("../assets/tiled_base64.tmx"))),
                "assets/tiled_base64.tmx",
                &mut NoopResourceCache,
            )
        })
    });
    c.bench_function("parse tiled_base64_zlib.tmx", |b| {
        b.iter(|| {
            Map::parse_reader(
                black_box(Cursor::new(include_str!("../assets/tiled_base64_zlib.tmx"))),
                "assets/tiled_base64_zlib.tmx",
                &mut NoopResourceCache,
            )
        })
    });
}

criterion_group!(benches, map_loading);
criterion_main!(benches);
