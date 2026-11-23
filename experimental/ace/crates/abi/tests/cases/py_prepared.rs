use std::path::PathBuf;
use std::sync::OnceLock;

use ace_abi::py_session::PySession;
use ace_abi::rust_provider::{
    ensure_python_dll_path, load_prepared_py_bridge, open_session_from_build, F0Session,
    RustProviderBuild,
};

fn provider() -> &'static RustProviderBuild {
    static PROVIDER: OnceLock<RustProviderBuild> = OnceLock::new();
    PROVIDER.get_or_init(|| {
        let root = std::env::var_os("ACE_AV_PROVIDER_ROOT")
            .map(PathBuf::from)
            .expect("foreign.prepared: run cargo t ace-python --prepare-only");
        load_prepared_py_bridge(&root).expect("source-bound Python provider")
    })
}

pub(super) fn py_session() -> PySession {
    PySession::from_prepared(provider()).expect("open prepared Python session")
}

pub(super) fn py_f0_session() -> F0Session {
    ensure_python_dll_path();
    open_session_from_build(provider(), 1).expect("open prepared F0 Python session")
}
