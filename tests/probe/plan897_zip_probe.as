// plan897_zip_probe.as — PLAN-897 zip natives VM 轨端到端探针（Z-01..Z-04）。
// 前置：python tests/probe/plan897_make_fixtures.py（生成 %TEMP%/plan897/zipfx.zip）。
// 判据双门：%TEMP%/plan897-zip-probe.log 的 PASS/FAIL 计数 + 进程退出码
// （fails>0 时以非法数组读主动终止——非零退出）。
// 运行：auto tests/probe/plan897_zip_probe.as
//
// 形态纪律：断言值全部为字面量/常量比较（find/==），不经 to_string 拼接
// （脚本上下文池串扰规避，N-003-3）。

fn log_path() str {
    return fs.join(Env.get("TEMP"), "plan897-zip-probe.log")
}

fn log(msg str) {
    var prev str = ""
    if file.exists(log_path()) {
        prev = file.read_text(log_path())
    }
    file.write_text(log_path(), prev + msg + "\n")
}

var fails int = 0

fn check(cond bool, name str) {
    if cond {
        log("PASS " + name)
    } else {
        fails = fails + 1
        log("FAIL " + name)
    }
}

fn main() {
    file.delete(log_path())
    log("PLAN897 start")
    let base str = fs.join(Env.get("TEMP"), "plan897")
    let zp str = fs.join(base, "zipfx.zip")

    // ── Z-01: list_json 形态与内容 ──
    let js str = zip.list_json(zp)
    check(js.find("[{\"name\":\"ascii.txt\"") == 0, "z01 list starts with ascii entry")
    check(js.find("\"name\":\"cjk.txt\"") > 0, "z01 contains cjk entry")
    check(js.find("\"name\":\"bin/logo.png\"") > 0, "z01 contains png entry")
    check(js.find("\"size\":19,") > 0, "z01 ascii size 19")
    check(js.find("\"size\":39,") > 0, "z01 cjk size 39")
    check(js.find("\"comp_size\":72,") > 0, "z01 png comp_size 72")
    check(js.find("\"method\":\"Deflated\"") > 0, "z01 deflated method present")
    check(js.find("\"method\":\"Stored\"") > 0, "z01 stored method present")
    // JSON 可被 json.parse 消费
    var parsed_ok bool = false
    try {
        let v = json.parse(js)
        // json.parse 数组 len()/索引有在册缺陷 F-8——按 for-in 计数
        var n int = 0
        for it in v {
            n = n + 1
        }
        if n == 4 {
            parsed_ok = true
        }
    } catch {
        parsed_ok = false
    }
    check(parsed_ok, "z01 json.parse consumes array")
    // 失败形态：缺文件
    let miss str = zip.list_json(fs.join(base, "missing.zip"))
    check(miss == "", "z01 missing file returns empty")

    // ── Z-02: read_text 逐字回环 + extract 字节精确 ──
    let a str = zip.read_text(zp, "ascii.txt", 1048576)
    check(a == "The quick brown fox", "z02 ascii roundtrip")
    let c str = zip.read_text(zp, "cjk.txt", 1048576)
    check(c == "山月不知心底事，水调歌头。", "z02 cjk roundtrip")
    let dest str = fs.join(base, "out_logo.png")
    file.delete(dest)
    let rc int = zip.extract_entry(zp, "bin/logo.png", dest, 1048576)
    check(rc == 0, "z02 extract rc 0")
    // 字节精确：dest sha256 与确定性常量对账
    var h str = ""
    if file.exists(dest) {
        h = hash.file_sha256(dest)
    }
    check(h == "8650fa3ad33b680c77fdf3067057857d38ade7f3ce31806131530d3a4d027052", "z02 extracted sha256 exact")

    // ── Z-03: max_bytes 硬闸 ──
    let capped str = zip.read_text(zp, "ascii.txt", 10)
    check(capped == "", "z03 read_text over cap returns empty")
    let dest2 str = fs.join(base, "out_capped.png")
    file.delete(dest2)
    let rc2 int = zip.extract_entry(zp, "bin/logo.png", dest2, 10)
    check(rc2 == -2, "z03 extract over cap returns -2")
    check(!file.exists(dest2), "z03 capped extract does not write dest")
    // 恰好等于上限 = 成功
    let exact str = zip.read_text(zp, "ascii.txt", 19)
    check(exact == "The quick brown fox", "z03 exact cap succeeds")

    // ── Z-04: 缺文件/缺条目/非 UTF-8 ──
    check(zip.read_text(zp, "nope.txt", 1048576) == "", "z04 missing entry empty")
    check(zip.read_text(zp, "latin.txt", 1048576) == "", "z04 non-utf8 entry empty")
    check(zip.read_text(zp, "ascii.txt", 0) == "", "z04 zero cap invalid")
    check(zip.extract_entry(zp, "nope.txt", dest2, 1048576) == -1, "z04 missing entry rc -1")

    log("PLAN897 done")
    // 双门之二：fails>0 → 主动触发非零退出（越界数组读）
    if fails > 0 {
        log("RESULT fails>0")
        let boom List<int> = List<int>.new([])
        log("boom-marker")
        log(boom[9999].to_string())
        return
    }
    log("RESULT all-pass")
}
