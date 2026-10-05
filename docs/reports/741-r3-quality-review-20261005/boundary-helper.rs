use auto_ac_prototype::link;
use std::{
    env, fs,
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
mod atom_text {
    pub use auto_ac_prototype::atom_text::*;
}
#[allow(dead_code)]
mod exact_link {
    include!("exact-link.rs");
    pub fn probe_job(cmd: &mut std::process::Command) -> String {
        let mut c = cmd.spawn().unwrap();
        let j = job::JobGuard::for_child(&c);
        let err = unsafe { windows_sys::Win32::Foundation::GetLastError() };
        let out = format!("job_some={};last_error={}", j.is_some(), err);
        let _ = c.kill();
        let _ = c.wait();
        drop(j);
        out
    }
    pub fn probe(cmd: &mut std::process::Command, millis: u64) -> String {
        format!(
            "{:?}",
            run_with_deadline(cmd, std::time::Duration::from_millis(millis))
        )
    }
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentProcess() -> *mut core::ffi::c_void;
}
fn main() {
    let args: Vec<String> = env::args().collect();
    match args[1].as_str() {
        "pipe-child" => std::thread::sleep(Duration::from_secs(30)),
        "fast-parent" => {
            let c = Command::new(env::current_exe().unwrap())
                .arg("pipe-child")
                .stdin(Stdio::null())
                .spawn()
                .unwrap();
            fs::write(&args[2], c.id().to_string()).unwrap();
            std::thread::sleep(Duration::from_millis(400));
        }
        "fallback" | "public-fallback" | "job-probe" => {
            use windows_sys::Win32::Foundation::CloseHandle;
            use windows_sys::Win32::System::JobObjects::*;
            unsafe {
                let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
                assert!(!job.is_null());
                let info = JOBOBJECT_BASIC_UI_RESTRICTIONS {
                    UIRestrictionsClass: JOB_OBJECT_UILIMIT_HANDLES,
                };
                assert_ne!(
                    SetInformationJobObject(
                        job,
                        JobObjectBasicUIRestrictions,
                        &info as *const _ as *const core::ffi::c_void,
                        std::mem::size_of_val(&info) as u32
                    ),
                    0
                );
                assert_ne!(AssignProcessToJobObject(job, GetCurrentProcess()), 0);
                let pidfile = PathBuf::from(&args[2]).join("fallback-child.pid");
                if pidfile.exists() {
                    fs::remove_file(&pidfile).unwrap();
                }
                let mut cmd = Command::new(env::current_exe().unwrap());
                cmd.arg("fast-parent").arg(&pidfile);
                if args[1] == "job-probe" {
                    println!("{}", exact_link::probe_job(&mut cmd));
                    CloseHandle(job);
                    return;
                }
                let t = Instant::now();
                let r = if args[1] == "public-fallback" {
                    env::set_var("P741_REVIEW_CHILD_PID", &pidfile);
                    format!("{:?}", link::run_exe(std::path::Path::new(&args[3])))
                } else {
                    exact_link::probe(&mut cmd, 1000)
                };
                let pid = fs::read_to_string(&pidfile).unwrap();
                use windows_sys::Win32::System::Threading::{
                    OpenProcess, TerminateProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE,
                    PROCESS_TERMINATE,
                };
                let child = OpenProcess(
                    PROCESS_SYNCHRONIZE | PROCESS_TERMINATE,
                    0,
                    pid.parse().unwrap(),
                );
                let open_error = windows_sys::Win32::Foundation::GetLastError();
                let wait = if child.is_null() {
                    u32::MAX
                } else {
                    WaitForSingleObject(child, 0)
                };
                println!("child_handle={child:?};open_error={open_error};wait_status={wait}");
                let alive = !child.is_null() && wait == 258;
                println!("mode={};deadline_ms={};elapsed_ms={};result={r};child_pid={pid};child_alive_after_return={alive}",args[1],if args[1]=="public-fallback"{60000}else{1000},t.elapsed().as_millis());
                if alive {
                    TerminateProcess(child, 99);
                    WaitForSingleObject(child, 2000);
                }
                if !child.is_null() {
                    CloseHandle(child);
                }
                CloseHandle(job);
            }
        }
        "locked-cleanup" => {
            use std::os::windows::fs::OpenOptionsExt;
            let d = PathBuf::from(&args[2]);
            fs::create_dir_all(&d).unwrap();
            let exe = d.join("tx.exe");
            let obj = d.join("tx.obj");
            let receipt = d.join("tx.ac-link.txt");
            fs::copy(&args[3], &exe).unwrap();
            fs::copy(&args[4], &obj).unwrap();
            fs::write(&receipt, b"old receipt").unwrap();
            let oldexe = fs::read(&exe).unwrap();
            let oldobj = fs::read(&obj).unwrap();
            let tag = link::staging_tag();
            let tmp_exe = d.join(format!("tx.exe.tmp-{tag}"));
            let tmp_obj = d.join(format!("tx.obj.tmp-{tag}"));
            let tmp_receipt = d.join(format!("tx.ac-link.txt.tmp-{tag}"));
            fs::copy(&exe, &tmp_exe).unwrap();
            fs::copy(&obj, &tmp_obj).unwrap();
            fs::create_dir(&tmp_receipt).unwrap();
            let lock = fs::OpenOptions::new()
                .read(true)
                .share_mode(1)
                .open(&tmp_exe)
                .unwrap();
            let stage = link::StagedLink {
                tmp_exe: tmp_exe.clone(),
                exe: exe.clone(),
                linker: PathBuf::from("rust-lld"),
                entry_symbol: "ac_start".into(),
            };
            let result = link::publish_artifacts(&stage, &tmp_obj, &obj);
            println!("result={result:?}\nstaged_exe_left={};staged_exe_path={};old_exe_unchanged={};old_obj_unchanged={};old_receipt_unchanged={};user_directory_preserved={}",tmp_exe.is_file(),tmp_exe.display(),fs::read(&exe).unwrap()==oldexe,fs::read(&obj).unwrap()==oldobj,fs::read(&receipt).unwrap()==b"old receipt",tmp_receipt.is_dir());
            drop(lock);
            fs::remove_file(&tmp_exe).unwrap();
        }
        _ => panic!("bad mode"),
    }
}
