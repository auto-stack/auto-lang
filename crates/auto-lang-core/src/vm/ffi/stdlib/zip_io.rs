// zip_io.rs — PLAN-897：VM zip native 三件套（list_json / read_text /
// extract_entry）。容器与解压由 zip 3.0 crate 承载（miniz_oxide 后端）。
// 错误形态契约（docs/specs/auto-vm/zip-natives.md）：
//   list_json      成功 = 中央目录顺序 JSON 数组；任何失败 = ""
//   read_text      成功 = 条目严格 UTF-8 文本；失败（缺文件/缺条目/非
//                  UTF-8/超 max_bytes/IO）= ""
//   extract_entry  0 = 成功；-1 = 一般失败；-2 = 解压字节超 max_bytes
// max_bytes ≤ 0 为非法参数（""/-1）——预算必填，native 读侧 take() 硬闸，
// 不信任 ZIP 头声明大小（解压炸弹防线）。
use super::request_root::resolve_request_file_path;

fn json_escape_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

fn open_archive(path: &str) -> Result<zip::ZipArchive<std::fs::File>, String> {
    let resolved = resolve_request_file_path(path)
        .map_err(|e| format!("zip open failed: {} - {}", path, e))?;
    let f = std::fs::File::open(&resolved)
        .map_err(|e| format!("zip open failed: {} - {}", resolved.display(), e))?;
    zip::ZipArchive::new(f).map_err(|e| format!("zip archive invalid: {} - {}", path, e))
}

/// `zip.list_json(path) -> String`
/// 中央目录顺序 JSON 数组：[{"name":..,"size":..,"comp_size":..,"method":..}]。
/// 失败返回 ""。
#[auto_macros::rust_fn("zip.list_json")]
pub fn zip_list_json(path: String) -> String {
    let mut archive = match open_archive(&path) {
        Ok(a) => a,
        Err(_) => return String::new(),
    };
    let mut out = String::from("[");
    for i in 0..archive.len() {
        let entry = match archive.by_index(i) {
            Ok(e) => e,
            Err(_) => return String::new(),
        };
        if i > 0 {
            out.push(',');
        }
        out.push_str("{\"name\":\"");
        out.push_str(&json_escape_str(entry.name()));
        out.push_str("\",\"size\":");
        out.push_str(&entry.size().to_string());
        out.push_str(",\"comp_size\":");
        out.push_str(&entry.compressed_size().to_string());
        out.push_str(",\"method\":\"");
        out.push_str(&format!("{:?}", entry.compression()));
        out.push_str("\"}");
    }
    out.push(']');
    out
}

/// `zip.read_text(path, name, max_bytes) -> String`
/// 条目解压并严格 UTF-8 解码；任何失败（含超限）返回 ""。
#[auto_macros::rust_fn("zip.read_text")]
pub fn zip_read_text(path: String, name: String, max_bytes: i32) -> String {
    if max_bytes <= 0 {
        return String::new();
    }
    let mut archive = match open_archive(&path) {
        Ok(a) => a,
        Err(_) => return String::new(),
    };
    let mut entry = match archive.by_name(&name) {
        Ok(e) => e,
        Err(_) => return String::new(),
    };
    let mut bytes = Vec::with_capacity((max_bytes as usize).min(1 << 20));
    let mut limited = std::io::Read::take(&mut entry, max_bytes as u64);
    if std::io::Read::read_to_end(&mut limited, &mut bytes).is_err() {
        return String::new();
    }
    // 超限判别：读满 limit 且源未到 EOF → 截断发生（超 max_bytes）
    if bytes.len() == max_bytes as usize {
        let mut probe = [0u8; 1];
        let n = std::io::Read::read(&mut entry, &mut probe).unwrap_or(0);
        if n != 0 {
            return String::new();
        }
    }
    String::from_utf8(bytes).unwrap_or_default()
}

/// `zip.extract_entry(path, name, dest, max_bytes) -> i32`
/// 条目解压字节写入 dest（截断语义）。0 成功；-1 一般失败；
/// -2 解压字节超 max_bytes。
#[auto_macros::rust_fn("zip.extract_entry")]
pub fn zip_extract_entry(path: String, name: String, dest: String, max_bytes: i32) -> i32 {
    if max_bytes <= 0 {
        return -1;
    }
    let resolved_dest = match resolve_request_file_path(&dest) {
        Ok(p) => p,
        Err(_) => return -1,
    };
    let mut archive = match open_archive(&path) {
        Ok(a) => a,
        Err(_) => return -1,
    };
    let mut entry = match archive.by_name(&name) {
        Ok(e) => e,
        Err(_) => return -1,
    };
    let mut bytes = Vec::with_capacity((max_bytes as usize).min(1 << 20));
    let mut limited = std::io::Read::take(&mut entry, max_bytes as u64);
    if std::io::Read::read_to_end(&mut limited, &mut bytes).is_err() {
        return -1;
    }
    if bytes.len() == max_bytes as usize {
        let mut probe = [0u8; 1];
        let n = std::io::Read::read(&mut entry, &mut probe).unwrap_or(0);
        if n != 0 {
            return -2;
        }
    }
    match std::fs::write(&resolved_dest, &bytes) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

#[cfg(test)]
mod plan897_tests {
    use super::*;
    use std::io::Write as IoWrite;

    /// 生成确定性 fixture：stored 文本 + deflate 文本（含 CJK）+ deflate 二进制。
    fn make_fixture_at(dir: &std::path::Path, name: &str) -> (std::path::PathBuf, Vec<(String, Vec<u8>)>) {
        let path = dir.join(name);
        let entries = vec![
            ("ascii.txt".to_string(), b"The quick brown fox".to_vec()),
            ("cjk.txt".to_string(), "山月不知心底事，水调歌头。".as_bytes().to_vec()),
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
        let stored: zip::write::SimpleFileOptions = opts.compression_method(zip::CompressionMethod::Stored);
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
        assert_eq!(zip_list_json(dir.join("missing.zip").to_string_lossy().to_string()), "");
    }

    #[test]
    fn plan897_read_text_roundtrip_and_caps() {
        let dir = std::env::temp_dir().join("plan897-test");
        std::fs::create_dir_all(&dir).unwrap();
        let (zp, entries) = make_fixture_at(&dir, "fx_read.zip");
        let p = zp.to_string_lossy().to_string();
        // 逐字回环（ASCII + CJK）
        assert_eq!(zip_read_text(p.clone(), "ascii.txt".into(), 1 << 20), "The quick brown fox");
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
        let rc2 = zip_extract_entry(p.clone(), "bin/logo.png".into(), dest.to_string_lossy().to_string(), 10);
        assert_eq!(rc2, -2, "cap rc");
        // 缺条目 → -1
        let rc3 = zip_extract_entry(p, "nope.bin".into(), dest.to_string_lossy().to_string(), 1 << 20);
        assert_eq!(rc3, -1, "missing rc");
    }
}
