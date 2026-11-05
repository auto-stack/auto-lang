//! VM adapters over the shared, bounded native Rust ZIP implementation.
use super::request_root::resolve_request_file_path;
#[auto_macros::rust_fn("zip.list_json")]
pub fn zip_list_json(path: String) -> String {
    let Ok(path) = resolve_request_file_path(&path) else {
        return String::new();
    };
    ::a2r_std::zip::list_json(&path.to_string_lossy())
}
#[auto_macros::rust_fn("zip.read_text")]
pub fn zip_read_text(path: String, name: String, max_bytes: i32) -> String {
    let Ok(path) = resolve_request_file_path(&path) else {
        return String::new();
    };
    ::a2r_std::zip::read_text(&path.to_string_lossy(), &name, max_bytes as i64)
}
#[auto_macros::rust_fn("zip.extract_entry")]
pub fn zip_extract_entry(path: String, name: String, dest: String, max_bytes: i32) -> i32 {
    let (Ok(path), Ok(dest)) = (
        resolve_request_file_path(&path),
        resolve_request_file_path(&dest),
    ) else {
        return -1;
    };
    ::a2r_std::zip::extract_entry(
        &path.to_string_lossy(),
        &name,
        &dest.to_string_lossy(),
        max_bytes as i64,
    ) as i32
}
#[cfg(test)]
mod plan897_tests {
    use super::*;
    use std::io::Write as IoWrite;

    /// 生成确定性 fixture：stored 文本 + deflate 文本（含 CJK）+ deflate 二进制。
    fn make_fixture_at(
        dir: &std::path::Path,
        name: &str,
    ) -> (std::path::PathBuf, Vec<(String, Vec<u8>)>) {
        let path = dir.join(name);
        let entries = vec![
            ("ascii.txt".to_string(), b"The quick brown fox".to_vec()),
            (
                "cjk.txt".to_string(),
                "山月不知心底事，水调歌头。".as_bytes().to_vec(),
            ),
            ("bin/logo.png".to_string(), {
                let mut v = vec![0x89u8, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
                for i in 0..64u32 {
                    v.push((i % 251) as u8);
                }
                v
            }),
        ];
        let file = std::fs::File::create(&path).unwrap();
        let mut w = zip::ZipWriter::new(file);
        let opts: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default();
        // 条目1: deflate
        w.start_file("ascii.txt", opts).unwrap();
        w.write_all(&entries[0].1).unwrap();
        // 条目2: deflate（默认方法）
        w.start_file("cjk.txt", opts).unwrap();
        w.write_all(&entries[1].1).unwrap();
        // 条目3: stored
        let stored: zip::write::SimpleFileOptions =
            opts.compression_method(zip::CompressionMethod::Stored);
        w.start_file("bin/logo.png", stored).unwrap();
        w.write_all(&entries[2].1).unwrap();
        w.finish().unwrap();
        (path, entries)
    }

    #[test]
    fn plan897_list_json_shape() {
        let dir = std::env::temp_dir().join("plan897-test");
        std::fs::create_dir_all(&dir).unwrap();
        let (zp, _entries) = make_fixture_at(&dir, "fx_list.zip");
        let entries = _entries;
        let js = zip_list_json(zp.to_string_lossy().to_string());
        assert!(js.starts_with("[{") && js.ends_with("}]"), "shape: {}", js);
        assert!(js.contains("\"name\":\"ascii.txt\""), "names: {}", js);
        assert!(js.contains("\"name\":\"cjk.txt\""), "names: {}", js);
        assert!(js.contains("\"name\":\"bin/logo.png\""), "names: {}", js);
        // size 与源字节一致
        let a0 = entries[0].1.len().to_string();
        assert!(js.contains(&format!("\"size\":{},", a0)), "size0 in {}", js);
        // method 字符串：Stored 与 Deflated 均出现
        assert!(js.contains("Stored"), "stored method in {}", js);
        assert!(js.contains("Deflated"), "deflate method in {}", js);
        // 失败形态
        assert_eq!(
            zip_list_json(dir.join("missing.zip").to_string_lossy().to_string()),
            ""
        );
    }

    #[test]
    fn plan897_read_text_roundtrip_and_caps() {
        let dir = std::env::temp_dir().join("plan897-test");
        std::fs::create_dir_all(&dir).unwrap();
        let (zp, entries) = make_fixture_at(&dir, "fx_read.zip");
        let p = zp.to_string_lossy().to_string();
        // 逐字回环（ASCII + CJK）
        assert_eq!(
            zip_read_text(p.clone(), "ascii.txt".into(), 1 << 20),
            "The quick brown fox"
        );
        assert_eq!(
            zip_read_text(p.clone(), "cjk.txt".into(), 1 << 20),
            "山月不知心底事，水调歌头。"
        );
        // 非法参数 / 缺条目
        assert_eq!(zip_read_text(p.clone(), "ascii.txt".into(), 0), "");
        assert_eq!(zip_read_text(p.clone(), "nope.txt".into(), 1 << 20), "");
        // max_bytes 硬闸：limit < 实际 → ""
        assert_eq!(zip_read_text(p.clone(), "ascii.txt".into(), 10), "");
        // max_bytes == 实际 → 恰好成功
        assert_eq!(
            zip_read_text(p, "ascii.txt".into(), entries[0].1.len() as i32),
            "The quick brown fox"
        );
    }

    #[test]
    fn plan897_extract_entry_bytes_exact() {
        let dir = std::env::temp_dir().join("plan897-test");
        std::fs::create_dir_all(&dir).unwrap();
        let (zp, entries) = make_fixture_at(&dir, "fx_extract.zip");
        let p = zp.to_string_lossy().to_string();
        let dest = dir.join("out_logo.png");
        let rc = zip_extract_entry(
            p.clone(),
            "bin/logo.png".into(),
            dest.to_string_lossy().to_string(),
            1 << 20,
        );
        assert_eq!(rc, 0, "extract rc");
        let back = std::fs::read(&dest).unwrap();
        assert_eq!(back, entries[2].1, "bytes exact");
        // 超限 → -2（与 -1 区分）
        let rc2 = zip_extract_entry(
            p.clone(),
            "bin/logo.png".into(),
            dest.to_string_lossy().to_string(),
            10,
        );
        assert_eq!(rc2, -2, "cap rc");
        // 缺条目 → -1
        let rc3 = zip_extract_entry(
            p,
            "nope.bin".into(),
            dest.to_string_lossy().to_string(),
            1 << 20,
        );
        assert_eq!(rc3, -1, "missing rc");
    }
}
