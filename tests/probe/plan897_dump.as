fn log_path() str {
    return fs.join(Env.get("TEMP"), "zipdump.log")
}
fn main() {
    let base str = fs.join(Env.get("TEMP"), "plan897")
    let js str = zip.list_json(fs.join(base, "zipfx.zip"))
    var ok bool = false
    var msg str = "parse-raise"
    try {
        let v = json.parse(js)
        let n int = v.len()
        if n == 4 {
            ok = true
        }
        msg = "len-not-4"
    } catch {
        ok = false
        msg = "raised"
    }
    file.write_text(log_path(), "ok-marker")
    if ok {
        file.write_text(log_path(), "PASS len4
")
    } else {
        file.write_text(log_path(), "FAIL " + msg + "
")
    }
}
