//! Test fault seams must restore caller-owned tool paths, including on panic.
pub(super) struct RestoreEnv(Vec<(&'static str, Option<std::ffi::OsString>)>);

impl RestoreEnv {
    pub(super) fn capture(names: &[&'static str]) -> Self {
        Self(
            names
                .iter()
                .map(|name| (*name, std::env::var_os(name)))
                .collect(),
        )
    }

    pub(super) fn restore(&self) {
        for (name, value) in &self.0 {
            if let Some(value) = value {
                std::env::set_var(name, value);
            } else {
                std::env::remove_var(name);
            }
        }
    }
}

impl Drop for RestoreEnv {
    fn drop(&mut self) {
        self.restore();
    }
}

#[test]
fn caller_values_and_absence_survive_fault_unwind() {
    let names = [
        "AUTO_LANG_ENV_RESTORE_888_PRESENT",
        "AUTO_LANG_ENV_RESTORE_888_ABSENT",
    ];
    let _original = RestoreEnv::capture(&names);
    std::env::set_var(names[0], "caller-owned");
    std::env::remove_var(names[1]);
    assert!(std::panic::catch_unwind(|| {
        let _restore = RestoreEnv::capture(&names);
        std::env::set_var(names[0], "fault");
        std::env::set_var(names[1], "temporary");
        panic!("intentional test fault");
    })
    .is_err());
    assert_eq!(std::env::var(names[0]).unwrap(), "caller-owned");
    assert!(std::env::var_os(names[1]).is_none());
}
