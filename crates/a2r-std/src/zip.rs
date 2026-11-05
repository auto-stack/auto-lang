// PLAN-897/956：共享纯 Rust ZIP 三件套（list_json / read_text /
// extract_entry）。容器与解压由 zip 3.0 crate 承载（miniz_oxide 后端）。
// 错误形态契约（docs/specs/auto-vm/zip-natives.md）：
//   list_json      成功 = 中央目录顺序 JSON 数组；任何失败 = ""
//   read_text      成功 = 条目严格 UTF-8 文本；失败（缺文件/缺条目/非
//                  UTF-8/超 max_bytes/IO）= ""
//   extract_entry  0 = 成功；-1 = 一般失败；-2 = 解压字节超 max_bytes
// max_bytes ≤ 0 为非法参数（""/-1）——预算必填，native 读侧 take() 硬闸，
// 不信任 ZIP 头声明大小（解压炸弹防线）。

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
    let resolved = std::path::PathBuf::from(path);
    let f = std::fs::File::open(&resolved)
        .map_err(|e| format!("zip open failed: {} - {}", resolved.display(), e))?;
    zip::ZipArchive::new(f).map_err(|e| format!("zip archive invalid: {} - {}", path, e))
}

/// `zip.list_json(path) -> String`
/// 中央目录顺序 JSON 数组：[{"name":..,"size":..,"comp_size":..,"method":..}]。
/// 失败返回 ""。
fn zip_list_json(path: String) -> String {
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
fn zip_read_text(path: String, name: String, max_bytes: i32) -> String {
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
fn zip_extract_entry(path: String, name: String, dest: String, max_bytes: i32) -> i32 {
    if max_bytes <= 0 {
        return -1;
    }
    let resolved_dest = std::path::PathBuf::from(dest);
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

pub fn list_json(path: &str) -> String {
    zip_list_json(path.to_string())
}
pub fn read_text(path: &str, name: &str, max_bytes: i64) -> String {
    let Ok(limit) = i32::try_from(max_bytes) else {
        return String::new();
    };
    zip_read_text(path.to_string(), name.to_string(), limit)
}
pub fn extract_entry(path: &str, name: &str, dest: &str, max_bytes: i64) -> i64 {
    let Ok(limit) = i32::try_from(max_bytes) else {
        return -1;
    };
    zip_extract_entry(path.to_string(), name.to_string(), dest.to_string(), limit) as i64
}

#[cfg(test)]
mod native_tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn utf8_binary_and_budget_failures_use_real_archive_bytes() {
        let dir = std::env::temp_dir().join(format!("rust-zip-{}", crate::uuid()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("book.zip");
        let mut writer = ::zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
        let options = ::zip::write::SimpleFileOptions::default()
            .compression_method(::zip::CompressionMethod::Deflated);
        writer.start_file("章.txt", options).unwrap();
        writer.write_all("中文😀".as_bytes()).unwrap();
        writer.start_file("image.bin", options).unwrap();
        writer.write_all(&[0xff, 0, 0x80]).unwrap();
        writer.finish().unwrap();
        let path = path.to_str().unwrap();
        let metadata: serde_json::Value = serde_json::from_str(&list_json(path)).unwrap();
        assert_eq!(metadata[0]["name"], "章.txt");
        assert_eq!(metadata[0]["size"], 10);
        assert_eq!(read_text(path, "章.txt", 10), "中文😀");
        assert_eq!(read_text(path, "章.txt", 9), "");
        assert_eq!(read_text(path, "image.bin", 10), "");
        assert_eq!(read_text(path, "章.txt", i64::MAX), "");
        let dest = dir.join("dest");
        std::fs::write(&dest, b"keep").unwrap();
        assert_eq!(extract_entry(path, "章.txt", dest.to_str().unwrap(), 9), -2);
        assert_eq!(std::fs::read(&dest).unwrap(), b"keep");
        assert_eq!(
            extract_entry(path, "image.bin", dest.to_str().unwrap(), 3),
            0
        );
        assert_eq!(std::fs::read(&dest).unwrap(), [0xff, 0, 0x80]);
        std::fs::remove_file(dest).unwrap();
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }
}
