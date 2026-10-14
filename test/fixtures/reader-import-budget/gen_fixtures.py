# PLAN-781 T-17: reader-import-budget fixture 生成器（确定性内容——无时间戳，
# 重跑逐字节一致；SHA256 边车清单 manifest.json 同步产出，供驱动做逐章
# 逐字核对而不复刻切章算法）。
#
# 产物（.gitignore 排除大文件；manifest.json 提交）：
#   book-10mib.txt   10MiB 中英混合多章 TXT（第N章 章头）
#   book-10mib.md    10MiB 中英混合多章 MD（# 第N章 章头）
#   book-small.txt   256KiB 种子书（非空库控制）
#   book-bad.bin     非法 UTF-8 控制文件
#   manifest.json    每章 {title, body_sha256, body_chars} + 文件 sha256
import hashlib
import json
import os

HERE = os.path.dirname(os.path.abspath(__file__))

CN_SENT = "长文本导入预算验收正文句，混合ASCII token p781 与中文标点，供十兆位档位逐字核对。"
EN_SENT = "The mixed-language verification line packs ASCII words and digits 0123456789 for byte-level checking. "


def chapters_for(target_bytes, fmt):
    """确定性生成 (heading, body) 章列表，总字节 ≤ target。"""
    out = []
    total = 0
    ch = 0
    while total < target_bytes:
        ch += 1
        title = f"第{ch}章 验收章节之{ch}"
        head = (f"# {title}\n" if fmt == "md" else f"{title}\n")
        body_lines = [f"{CN_SENT} 第{ch}章第{i}行。" for i in range(30)]
        body_lines += [f"{EN_SENT} Chapter {ch} line {i}." for i in range(30)]
        body = "\n".join(body_lines)
        seg = head + body + "\n"
        b = seg.encode("utf-8")
        if total + len(b) > target_bytes:
            # 尾章：按合法 UTF-8 边界截 body，空行补足字节数。
            room = target_bytes - total
            head_b = head.encode("utf-8")
            body_room = room - len(head_b)
            if body_room < 64:
                break
            cut = body.encode("utf-8")[: body_room].decode("utf-8", "ignore")
            body = cut + "\n" * (body_room - len(cut.encode("utf-8")))
            b = head_b + body.encode("utf-8")
        out.append((title, head, body))
        total += len(b)
    return out, total


def build(path, fmt, target):
    chapters, total = chapters_for(target, fmt)
    parts = []
    manifest = {"file": os.path.basename(path), "fmt": fmt, "bytes": total,
                "chapter_count": len(chapters), "chapters": []}
    for i, (title, head, body) in enumerate(chapters, 1):
        parts.append(head.encode("utf-8"))
        parts.append(body.encode("utf-8"))
        parts.append(b"\n")
        # Reader make_chapter 对 body 做 trim——清单哈希按同一口径。
        trimmed = body.strip()
        manifest["chapters"].append({
            "number": i,
            "title": title,
            "body_sha256": hashlib.sha256(trimmed.encode("utf-8")).hexdigest(),
            "body_chars": len(trimmed),
        })
    blob = b"".join(parts)
    # 末尾补换行到目标字节数（章后换行已含；不足处补 \n）。
    blob += b"\n" * max(0, target - len(blob))
    blob = blob[:target]
    manifest["bytes"] = len(blob)
    manifest["sha256"] = hashlib.sha256(blob).hexdigest()
    with open(path, "wb") as f:
        f.write(blob)
    return manifest


def main():
    mans = []
    m = build(os.path.join(HERE, "book-10mib.txt"), "txt", 10 * 1024 * 1024)
    mans.append(m)
    m = build(os.path.join(HERE, "book-10mib.md"), "md", 10 * 1024 * 1024)
    mans.append(m)
    m = build(os.path.join(HERE, "book-small.txt"), "txt", 256 * 1024)
    mans.append(m)
    # 低预算实例的非空库种子（4KiB ≈ <0.2M 步——1M 预算档内可成功）。
    m = build(os.path.join(HERE, "book-tiny.txt"), "txt", 4 * 1024)
    mans.append(m)
    # 非法 UTF-8：合法前缀 + 0xFF 0xFE 孤立字节 + 合法尾。
    bad = ("VALID-PREFIX 验收前缀。\n第1章 坏编码\n").encode("utf-8")
    bad += b"\xFF\xFE" + ("\nVALID-SUFFIX\n".encode("utf-8") * 100)
    bad_path = os.path.join(HERE, "book-bad.bin")
    with open(bad_path, "wb") as f:
        f.write(bad)
    mans.append({"file": "book-bad.bin", "bytes": len(bad),
                 "sha256": hashlib.sha256(bad).hexdigest(),
                 "invalid_utf8": True})
    with open(os.path.join(HERE, "manifest.json"), "w", encoding="utf-8") as f:
        json.dump({"fixtures": mans}, f, ensure_ascii=False, indent=1)
    for m in mans:
        print(f"{m['file']}: {m['bytes']} bytes sha256={m['sha256'][:16]}… "
              f"chapters={m.get('chapter_count', '-')}")


if __name__ == "__main__":
    main()
