//! Plan 673 T-03 / P670-D1: dual-track byte parity for `read_text_range`.
//!
//! The AutoVM native (`shim_file_read_text_range`, nat#1016) and the a2r-std
//! mirror (`a2r_std::fs::read_text_range`) must emit BYTE-IDENTICAL JSON for
//! the same file + params across the whole boundary matrix (design §5):
//! normal reads, true EOF (`next_offset: null` only when the chunk reaches
//! the byte tail — a disk-tail short read must not fake EOF), offset past
//! end, empty file, and multi-byte UTF-8 edges (never a half char).

use std::fs;

fn vm_track(path: &str, offset: i32, limit: i32) -> String {
    crate::vm::ffi::stdlib::shim_file_read_text_range(path.to_string(), offset, limit)
}

fn a2r_track(path: &str, offset: usize, limit: usize) -> String {
    a2r_std::fs::read_text_range(path, offset, limit)
}

#[test]
fn read_text_range_dual_track_byte_parity() {
    let dir = std::env::temp_dir().join("auto_lang_rtr_parity_673");
    fs::create_dir_all(&dir).unwrap();
    let ascii = {
        let p = dir.join("ascii.txt");
        fs::write(&p, "abcdefghij").unwrap();
        p
    };
    let cjk = {
        let p = dir.join("cjk.txt");
        fs::write(&p, "你好世界").unwrap();
        p
    };
    let empty = {
        let p = dir.join("empty.txt");
        fs::write(&p, "").unwrap();
        p
    };
    let missing = dir.join("definitely_missing.txt");

    // (file, offset, limit) — every case runs through BOTH tracks.
    let cases: &[(&std::path::Path, usize, usize)] = &[
        (&ascii, 0, 4),
        (&ascii, 2, 4),
        (&ascii, 6, 10),
        (&ascii, 9, 1),
        (&ascii, 10, 1),  // offset == total → EOF shape
        (&ascii, 100, 4), // offset past end → EOF shape, real total
        (&cjk, 0, 4),     // limit backs off the 3-byte boundary
        (&cjk, 2, 4),     // offset walks back to the boundary
        (&cjk, 3, 3),
        (&cjk, 9, 10),    // CJK EOF → null next_offset
        (&cjk, 0, 12),    // whole file, exact limit
        (&empty, 0, 10),
        (&empty, 5, 10),
        (&missing, 0, 4), // IO error shape
    ];
    for &(path, offset, limit) in cases {
        let path = path.to_str().unwrap();
        let vm = vm_track(path, offset as i32, limit as i32);
        let a2r = a2r_track(path, offset, limit);
        assert_eq!(vm, a2r, "track divergence at {path} offset={offset} limit={limit}");
    }

    // Spot-check the exact shapes on the VM track (the .at fixture pins the
    // same literals end-to-end through the VM pipeline).
    assert_eq!(
        vm_track(ascii.to_str().unwrap(), 2, 4),
        r#"{"text":"cdef","total":10,"next_offset":6}"#
    );
    assert_eq!(
        vm_track(ascii.to_str().unwrap(), 6, 10),
        r#"{"text":"ghij","total":10,"next_offset":null}"#
    );
    assert_eq!(
        vm_track(cjk.to_str().unwrap(), 0, 4),
        r#"{"text":"你","total":12,"next_offset":3}"#
    );

    // Parameter errors: the VM track takes i32, so negatives arrive from
    // .at code and must return the error shape (design §5.2). The a2r track
    // takes usize (negatives cannot be expressed) and documents the gap;
    // the limit==0 shape still agrees byte-for-byte.
    let err_shape = r#"{"text":"","total":-1,"next_offset":null}"#;
    assert_eq!(vm_track(ascii.to_str().unwrap(), -1, 4), err_shape);
    assert_eq!(vm_track(ascii.to_str().unwrap(), 0, 0), err_shape);
    assert_eq!(vm_track(ascii.to_str().unwrap(), 2, -5), err_shape);
    assert_eq!(a2r_track(ascii.to_str().unwrap(), 0, 0), err_shape);

    fs::remove_dir_all(&dir).ok();
}
