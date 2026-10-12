use auto_ac_prototype::link;
use std::{env, fs, path::{Path, PathBuf}, process::{Command, Stdio}, thread, time::{Duration, Instant}};
use std::os::windows::fs::OpenOptionsExt;

fn main() {
    let args: Vec<String> = env::args().collect();
    if let Some(out) = args.iter().find_map(|a| a.strip_prefix("/out:")) {
        // Stand-in linker creates its own partial output, then waits until
        // the observer holds a normal sharing lock before reporting failure.
        let p = PathBuf::from(out);
        fs::write(&p, b"partial-linker-output").unwrap();
        let ready = p.with_extension("lock-ready");
        let deadline = Instant::now() + Duration::from_secs(10);
        while !ready.exists() {
            assert!(Instant::now() < deadline, "observer never acquired lock");
            thread::sleep(Duration::from_millis(5));
        }
        if args.iter().any(|a| a == "/entry:timeout_probe") {
            thread::sleep(Duration::from_secs(120));
        }
        std::process::exit(42);
    }
    let me = env::current_exe().unwrap();
    match me.file_stem().unwrap().to_str().unwrap() {
        "tree-child" => { thread::sleep(Duration::from_secs(120)); return; }
        "tree-parent" => {
            let child = Command::new(me.with_file_name("tree-child.exe"))
                .stdin(Stdio::null()).spawn().unwrap();
            fs::write(me.with_file_name("tree-child.pid"), child.id().to_string()).unwrap();
            return;
        }
        _ => {}
    }
    if args[1] == "public-deadline" {
        let dir = Path::new(&args[2]); fs::create_dir_all(dir).unwrap();
        let parent = dir.join("tree-parent.exe");
        fs::copy(&me, &parent).unwrap();
        fs::copy(&me, dir.join("tree-child.exe")).unwrap();
        let started = Instant::now();
        let result = link::run_exe(&parent);
        println!("public_result={result:?}\nelapsed_ms={}", started.elapsed().as_millis());
        let pid = fs::read_to_string(dir.join("tree-child.pid")).unwrap();
        // Exact PID only, no global image-name cleanup or survivor killing.
        let started = Instant::now();
        loop {
            let output = Command::new("tasklist").args(["/FI", &format!("PID eq {}", pid.trim()), "/NH"])
                .output().unwrap();
            let alive = String::from_utf8_lossy(&output.stdout).contains("tree-child.exe");
            if !alive { println!("descendant_alive_after_poll=false\ndeath_watch_ms={}", started.elapsed().as_millis()); break; }
            if started.elapsed() > Duration::from_secs(5) { println!("descendant_alive_after_poll=true\npid={pid}"); break; }
            thread::sleep(Duration::from_millis(20));
        }
        return;
    }
    let dir = Path::new(&args[2]); fs::create_dir_all(dir).unwrap();
    // Baseline was produced by the real Cranelift/rust-lld CLI, not fake bytes.
    for ext in ["exe", "obj", "ac-link.txt"] {
        fs::copy(Path::new(&args[3]).join(format!("old.{ext}")), dir.join(format!("tx.{ext}"))).unwrap();
    }
    let exe = dir.join("tx.exe"); let obj = dir.join("tx.obj");
    let previous: Vec<_> = [&exe, &obj, &dir.join("tx.ac-link.txt")].iter().map(|p| fs::read(p).unwrap()).collect();
    let temp = dir.join(format!("tx.exe.tmp-{}", link::staging_tag()));
    let locked_path = temp.clone();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let (locked_tx, locked_rx) = std::sync::mpsc::channel();
    let locker = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !locked_path.is_file() {
            assert!(Instant::now() < deadline, "linker never wrote partial output");
            thread::sleep(Duration::from_millis(5));
        }
        let held = fs::OpenOptions::new().read(true).share_mode(3).open(&locked_path).unwrap();
        fs::write(locked_path.with_extension("lock-ready"), b"locked").unwrap();
        locked_tx.send(()).unwrap();
        release_rx.recv().unwrap(); drop(held);
    });
    let started = Instant::now();
    let entry = if args[1] == "timeout" { "timeout_probe" } else { "d_entry" };
    let result = link::link_object_staged(&me, Path::new("."), &obj, &exe, entry, &[]);
    locked_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    let diag = match result { Err(d) => d.render("", ""), Ok(_) => panic!("fake linker unexpectedly succeeded") };
    println!("diagnostic={diag}\nelapsed_ms={}\nstaged_exe_left={}\nstaged_exe_path={}\npath_in_diagnostic={}\ncleanup_in_diagnostic={}",
        started.elapsed().as_millis(), temp.exists(), temp.display(), diag.contains(&temp.display().to_string()), diag.contains("cleanup"));
    let remove_error = fs::remove_file(&temp).unwrap_err();
    println!("actual_remove_os_error={:?}", remove_error.raw_os_error());
    for (i, p) in [&exe, &obj, &dir.join("tx.ac-link.txt")].iter().enumerate() {
        println!("old_{i}_unchanged={}", previous[i] == fs::read(p).unwrap());
    }
    let status = Command::new(&exe).status().unwrap(); println!("old_exe_exit={:?}", status.code());
    release_tx.send(()).unwrap(); locker.join().unwrap();
    fs::remove_file(&temp).unwrap(); fs::remove_file(temp.with_extension("lock-ready")).unwrap();
    println!("released_cleanup_ok={}", !temp.exists());
}
