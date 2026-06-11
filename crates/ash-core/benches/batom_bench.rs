//! Batom performance benchmarks
//!
//! Measures encoding/decoding throughput for various Atom payload sizes.

use ash_core::pipeline::atom::{Atom, AtomType};
use ash_core::pipeline::atom_pipeline::AtomPipeline;
use ash_core::pipeline::atom_stream::AtomStream;
use ash_core::pipeline::batom::{decode_atom, decode_pipeline, encode_atom, encode_pipeline};
use auto_val::Value;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

// ── Helpers to build test data ─────────────────────────────

fn make_file_entry(name: &str, size: i32) -> Value {
    let mut obj = auto_val::Obj::new();
    obj.set("name", Value::str(name));
    obj.set("size", Value::Int(size));
    obj.set("type", Value::str("file"));
    obj.set("modified", Value::str("2026-06-11 14:30:00"));
    Value::Obj(obj)
}

fn make_file_list(n: usize) -> Atom {
    let entries: Vec<Value> = (0..n)
        .map(|i| make_file_entry(&format!("file_{:04}.txt", i), (i * 137) as i32))
        .collect();
    Atom::file_list(Value::Array(entries.into()))
}

// ── Benchmarks ─────────────────────────────────────────────

fn bench_encode_primitives(c: &mut Criterion) {
    let atom = Atom::new(Value::Int(42), AtomType::CountResult);
    c.bench_function("encode_int", |b| {
        b.iter(|| encode_atom(black_box(&atom)))
    });

    let atom = Atom::text("hello world");
    c.bench_function("encode_string", |b| {
        b.iter(|| encode_atom(black_box(&atom)))
    });

    let atom = Atom::new(Value::Float(3.14159), AtomType::Nothing);
    c.bench_function("encode_float", |b| {
        b.iter(|| encode_atom(black_box(&atom)))
    });
}

fn bench_decode_primitives(c: &mut Criterion) {
    let atom = Atom::new(Value::Int(42), AtomType::CountResult);
    let bytes = encode_atom(&atom).unwrap();

    c.bench_function("decode_int", |b| {
        b.iter(|| decode_atom(black_box(&bytes)))
    });

    let atom = Atom::text("hello world");
    let bytes = encode_atom(&atom).unwrap();
    c.bench_function("decode_string", |b| {
        b.iter(|| decode_atom(black_box(&bytes)))
    });
}

fn bench_encode_file_list(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode_file_list");
    for size in [10, 100, 1000] {
        let atom = make_file_list(size);
        group.bench_with_input(
            BenchmarkId::new("entries", size),
            &atom,
            |b, atom| {
                b.iter(|| encode_atom(black_box(atom)));
            },
        );
    }
    group.finish();
}

fn bench_decode_file_list(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_file_list");
    for size in [10, 100, 1000] {
        let atom = make_file_list(size);
        let bytes = encode_atom(&atom).unwrap();
        group.bench_with_input(
            BenchmarkId::new("entries", size),
            &bytes,
            |b, bytes| {
                b.iter(|| decode_atom(black_box(bytes)));
            },
        );
    }
    group.finish();
}

fn bench_roundtrip_file_list(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip_file_list");
    for size in [10, 100, 1000] {
        let atom = make_file_list(size);
        group.bench_with_input(
            BenchmarkId::new("entries", size),
            &atom,
            |b, atom| {
                b.iter(|| {
                    let bytes = encode_atom(black_box(atom)).unwrap();
                    let _ = decode_atom(black_box(&bytes)).unwrap();
                });
            },
        );
    }
    group.finish();
}

fn bench_pipeline_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline_roundtrip");

    // Text pipeline
    let pipeline = AtomPipeline::text("hello world from the shell");
    group.bench_function("text", |b| {
        b.iter(|| {
            let bytes = encode_pipeline(black_box(&pipeline)).unwrap();
            let _ = decode_pipeline(black_box(&bytes)).unwrap();
        })
    });

    // Atom pipeline
    let atom = make_file_list(100);
    let pipeline = AtomPipeline::from_atom(atom);
    group.bench_function("atom_100", |b| {
        b.iter(|| {
            let bytes = encode_pipeline(black_box(&pipeline)).unwrap();
            let _ = decode_pipeline(black_box(&bytes)).unwrap();
        })
    });

    // Stream pipeline
    let atoms: Vec<Atom> = (0..100)
        .map(|i| Atom::new(Value::Int(i), AtomType::FileEntry))
        .collect();
    let stream = AtomStream::new(atoms);
    let pipeline = AtomPipeline::from_stream(stream);
    group.bench_function("stream_100", |b| {
        b.iter(|| {
            let bytes = encode_pipeline(black_box(&pipeline)).unwrap();
            let _ = decode_pipeline(black_box(&bytes)).unwrap();
        })
    });

    group.finish();
}

fn bench_string_dedup(c: &mut Criterion) {
    // 1000 entries with many repeated strings
    let mut items = Vec::new();
    for i in 0..1000 {
        let mut obj = auto_val::Obj::new();
        obj.set("type", Value::str("file")); // repeated 1000x
        obj.set("status", Value::str("ok")); // repeated 1000x
        obj.set("ext", Value::str(if i % 3 == 0 { "rs" } else if i % 3 == 1 { "txt" } else { "md" }));
        obj.set("name", Value::str(&format!("file_{}", i)));
        items.push(Value::Obj(obj));
    }
    let atom = Atom::file_list(Value::Array(items.into()));

    c.bench_function("encode_dedup_1000", |b| {
        b.iter(|| encode_atom(black_box(&atom)))
    });
}

criterion_group!(
    benches,
    bench_encode_primitives,
    bench_decode_primitives,
    bench_encode_file_list,
    bench_decode_file_list,
    bench_roundtrip_file_list,
    bench_pipeline_roundtrip,
    bench_string_dedup,
);
criterion_main!(benches);
