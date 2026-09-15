// Reference: Rust-cookbook memory-mapping example, adapted to the safe
// anonymous-map API (file-backed Mmap::map is unsafe — out of Auto's safe
// surface; provenance note in memmap.at).
use memmap2::MmapOptions;

fn main() {
    let mut mmap = MmapOptions::new().len(4).map_anon().unwrap();
    mmap[0] = 42;
    let byte0 = mmap[0];
    let n = mmap.len();
    println!("len: {}, first byte: {}", n, byte0);
    assert_eq!(byte0, 42);
    assert_eq!(n, 4);
}
