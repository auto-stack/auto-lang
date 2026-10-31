fn log_path() str {
    return fs.join(Env.get("TEMP"), "zipmin.log")
}
fn main() {
    let h str = hash.sha256("abc")
    file.write_text(log_path(), "hash=" + h + "
")
    let js str = zip.list_json("nonexistent.zip")
    file.write_text(log_path(), "zip-ok js-len-marker
")
}
