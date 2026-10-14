#![allow(dead_code)]
#[path = "D:/autostack/auto-lang/crates/a2r-std/src/sqlite.rs"] mod sqlite;
#[path = "D:/autostack/auto-lang/crates/a2r-std/src/fs.rs"] mod fs;
fn main() {
    let db = sqlite::open(":memory:".to_string());
    let clone = db.clone();
    assert_eq!(db.exec("CREATE TABLE t (v TEXT)".to_string()), 0);
    assert_eq!(db.exec("BEGIN IMMEDIATE".to_string()), 0);
    assert_eq!(clone.exec("INSERT INTO t VALUES ('same connection')".to_string()), 1);
    assert_eq!(db.query("SELECT v FROM t".to_string()), vec![vec!["same connection".to_string()]]);
    assert!(db.exec("ROLLBACK") >= 0);
    assert!(clone.query("SELECT v FROM t").is_empty());
    assert!(clone.exec("BEGIN IMMEDIATE") >= 0);
    assert_eq!(db.exec("INSERT INTO t VALUES ('committed')"), 1);
    assert!(clone.exec("COMMIT") >= 0);
    assert_eq!(db.query("SELECT v FROM t"), vec![vec!["committed".to_string()]]);
    println!("D7: owned String inputs, clone visibility, rollback and commit PASS");
    let root = std::env::args().nth(1).expect("scratch directory");
    assert_eq!(fs::mkdir_all(&root), 0);
    std::fs::write(format!("{root}/z.txt"), "z").unwrap();
    std::fs::write(format!("{root}/a.txt"), "a").unwrap();
    std::fs::write(format!("{root}/.hidden"), "h").unwrap();
    assert_eq!(fs::dir_names(&root), vec!["a.txt".to_string(), "z.txt".to_string()]);
    assert!(fs::dir_names(&format!("{root}/missing")).is_empty());
    println!("D9: direct names sorted, hidden names skipped, missing directory empty PASS");
}
