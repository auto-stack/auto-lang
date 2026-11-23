use super::{ServiceError, Todo, MAX_TEXT_BYTES, MAX_TODOS};
use ace_av::modules::packages::PackageAvTarget;
use ace_av_core::contract::{ExecOutcome, Value, TEXT_RUNTIME_CAPABILITY};
use ace_av_core::function_code::SharedFunctionCode;
use ace_av_core::platform_io::{PlatformIoBackend, StreamKind, Win32PlatformIo};
use ace_av_core::{execute_program_text_platform, CancelFlag, ExecLimits, ProgramSnapshot};
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{
    AssemblyBackend, PackageAssemblyContext, ProviderChoice, TargetOs,
};
use ace_source_query::modules::packages::{
    assemble_stdlib_sys_inputs, compile_assembled_aggregates_inputs,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs::{self, Metadata};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

const SOURCE: &str = include_str!("storage.at");
const ENTRY: &str = "d_main";
const STORE_NAME: &str = "todo.json";
const STAGING_NAME: &str = "todo.json.tmp";
const FORMAT_VERSION: u8 = 1;
const MAX_SNAPSHOT_BYTES: u64 = 256 * 1024;
const ACCESS_DENIED: u32 = 5;

// The frozen sysplat module carries declarations for operations other than
// replace_atomic. The AV front door checks the whole linked product. This
// owner only implements memory and atomic replacement of the two fixed paths.
const STORAGE_IMPORTS: &[&str] = &[
    "platform.clock",
    "platform.entropy",
    "platform.env",
    "platform.file",
    "platform.file.path",
    "platform.io.stdin",
    "platform.mem",
    "platform.process",
    "platform.process.argv",
];

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Snapshot {
    version: u8,
    pub(super) next_id: i32,
    pub(super) todos: Vec<Todo>,
}

impl Snapshot {
    pub(super) fn new(next_id: i32, todos: Vec<Todo>) -> Self {
        Self {
            version: FORMAT_VERSION,
            next_id,
            todos,
        }
    }

    fn validate(&self) -> Result<(), ServiceError> {
        if self.version != FORMAT_VERSION {
            return Err(ServiceError::StorageVersion(self.version));
        }
        if self.todos.len() > MAX_TODOS
            || self
                .todos
                .iter()
                .any(|todo| todo.text.len() > MAX_TEXT_BYTES)
        {
            return Err(ServiceError::Budget("todo.snapshot-rows"));
        }
        if self.next_id < 4
            || self
                .todos
                .iter()
                .any(|todo| todo.id < 0 || todo.id >= self.next_id)
            || self.todos.windows(2).any(|rows| rows[0].id >= rows[1].id)
        {
            return Err(corrupt("invalid Todo IDs or next_id"));
        }
        Ok(())
    }
}

pub(super) struct StorageGrant {
    root: PathBuf,
    code: Vec<SharedFunctionCode>,
    imports: Vec<String>,
}

impl StorageGrant {
    pub(super) fn prepare(root: &Path, code_store: &Path) -> Result<Self, ServiceError> {
        if !cfg!(all(windows, target_arch = "x86_64")) || !root.is_absolute() {
            return Err(ServiceError::Unauthorized);
        }
        validate_root(root)?;
        let root = root.canonicalize().map_err(io_error)?;
        let mut context = PackageAssemblyContext::new(
            AssemblyBackend::Av,
            ProviderChoice::Auto,
            TargetOs::Windows,
        );
        let mut inputs = assemble_stdlib_sys_inputs(
            &[("main.at".into(), SOURCE.into())],
            "app::main",
            &mut context,
        )
        .map_err(storage_error)?;
        inputs.retain(|unit| {
            unit.package != "std"
                || matches!(
                    unit.id.as_str(),
                    "file" | "bytes" | "syscommon" | "sysplat_win" | "sysplat_lnx"
                )
        });
        let mut program = compile_assembled_aggregates_inputs(
            &inputs,
            "app::main",
            FrontBudget::WORK_OWNED_64MIB,
            &context,
        )
        .map_err(|errors| storage_error(format!("{errors:?}")))?;
        program
            .compiled
            .checked
            .require_capability(TEXT_RUNTIME_CAPABILITY);
        let target = PackageAvTarget::cold(code_store, &program).map_err(storage_error)?;
        if !target.current.globals().is_empty() {
            return Err(storage_error("storage program declares global cells"));
        }
        let code = target.current.code().to_vec();
        let imports = STORAGE_IMPORTS.iter().map(|name| (*name).into()).collect();
        Ok(Self {
            root,
            code,
            imports,
        })
    }

    pub(super) fn activate(&self) -> Result<Store<'_>, ServiceError> {
        validate_root(&self.root)?;
        let root = ActiveRoot::claim(&self.root)?;
        let path = self.root.join(STORE_NAME);
        let staging = self.root.join(STAGING_NAME);
        if path.to_str().is_none() || staging.to_str().is_none() {
            return Err(ServiceError::Unauthorized);
        }
        Ok(Store {
            _root: root,
            program: ProgramSnapshot::link(self.code.clone(), ENTRY, 0).map_err(storage_error)?,
            path,
            staging,
            imports: &self.imports,
        })
    }
}

pub(super) struct Store<'a> {
    _root: ActiveRoot<'a>,
    program: ProgramSnapshot,
    path: PathBuf,
    staging: PathBuf,
    imports: &'a [String],
}

struct ActiveRoot<'a>(&'a Path);

fn active_roots() -> &'static Mutex<BTreeSet<PathBuf>> {
    static ROOTS: OnceLock<Mutex<BTreeSet<PathBuf>>> = OnceLock::new();
    ROOTS.get_or_init(Mutex::default)
}

impl<'a> ActiveRoot<'a> {
    fn claim(root: &'a Path) -> Result<Self, ServiceError> {
        if !active_roots().lock().unwrap().insert(root.to_path_buf()) {
            return Err(ServiceError::Unauthorized);
        }
        Ok(Self(root))
    }
}

impl Drop for ActiveRoot<'_> {
    fn drop(&mut self) {
        active_roots().lock().unwrap().remove(self.0);
    }
}

impl Store<'_> {
    pub(super) fn load(&self) -> Result<Option<Snapshot>, ServiceError> {
        self.check_paths()?;
        let mut file = match fs::File::open(&self.path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(io_error(error)),
        };
        let mut bytes = Vec::new();
        (&mut file)
            .take(MAX_SNAPSHOT_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(io_error)?;
        check_size(bytes.len())?;
        let snapshot: Snapshot = serde_json::from_slice(&bytes).map_err(corrupt)?;
        snapshot.validate()?;
        Ok(Some(snapshot))
    }

    pub(super) fn save(&self, snapshot: &Snapshot) -> Result<(), ServiceError> {
        snapshot.validate()?;
        let body = serde_json::to_vec(snapshot).map_err(storage_error)?;
        check_size(body.len())?;
        self.check_paths()?;
        // A previous process may have exited before replacing the committed
        // file. Remove its staging name and create a fresh file exclusively.
        remove_staging(&self.staging)?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.staging)
            .map_err(io_error)?;
        let synchronized = file.write_all(&body).and_then(|()| file.sync_all());
        drop(file);
        let result = synchronized.map_err(io_error).and_then(|()| self.replace());
        if result.is_err() {
            remove_staging(&self.staging)?;
        }
        result
    }

    fn check_paths(&self) -> Result<(), ServiceError> {
        validate_root(self.path.parent().expect("data root"))?;
        validate_file(&self.path)?;
        validate_file(&self.staging)
    }

    fn replace(&self) -> Result<(), ServiceError> {
        let path = self.path.to_str().expect("validated data path");
        let staging = self.staging.to_str().expect("validated staging path");
        let mut platform = AtomicReplace {
            inner: Win32PlatformIo::default(),
            path: path.encode_utf16().collect(),
            staging: staging.encode_utf16().collect(),
            replaced: false,
        };
        let run = execute_program_text_platform(
            &self.program,
            &ExecLimits::default(),
            &CancelFlag::default(),
            self.imports,
            &[staging.as_bytes(), path.as_bytes()],
            None,
            &mut platform,
        );
        // Successful replacement is the commit point, including if the
        // interpreter subsequently runs out of fuel before returning.
        if platform.replaced {
            return Ok(());
        }
        let (outcome, _, summary) = run.map_err(storage_error)?;
        match outcome {
            ExecOutcome::Success {
                value: Value::I32(status),
                ..
            } => Err(storage_error(format!(
                "atomic replace returned {status}, OS error {}",
                summary.last_os_error
            ))),
            ExecOutcome::Resource { kind, .. } => Err(ServiceError::Budget(kind.code())),
            other => Err(storage_error(format!("{other:?}"))),
        }
    }
}

fn check_size(size: usize) -> Result<(), ServiceError> {
    if size as u64 > MAX_SNAPSHOT_BYTES {
        return Err(ServiceError::Budget("todo.snapshot-bytes"));
    }
    Ok(())
}

fn validate_root(path: &Path) -> Result<(), ServiceError> {
    let meta = fs::symlink_metadata(path).map_err(io_error)?;
    if !meta.is_dir() || reparse(&meta) {
        return Err(ServiceError::Unauthorized);
    }
    Ok(())
}

fn validate_file(path: &Path) -> Result<(), ServiceError> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_file() && !reparse(&meta) => Ok(()),
        Ok(_) => Err(ServiceError::Unauthorized),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_error(error)),
    }
}

fn remove_staging(path: &Path) -> Result<(), ServiceError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_error(error)),
    }
}

fn reparse(meta: &Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
    }
    #[cfg(not(windows))]
    {
        meta.file_type().is_symlink()
    }
}

fn corrupt(error: impl ToString) -> ServiceError {
    ServiceError::StorageCorrupt(error.to_string())
}

fn storage_error(error: impl ToString) -> ServiceError {
    ServiceError::Storage(error.to_string())
}

fn io_error(error: std::io::Error) -> ServiceError {
    if error.kind() == std::io::ErrorKind::PermissionDenied {
        ServiceError::Unauthorized
    } else {
        storage_error(error)
    }
}

struct AtomicReplace {
    inner: Win32PlatformIo,
    path: Vec<u16>,
    staging: Vec<u16>,
    replaced: bool,
}

impl PlatformIoBackend for AtomicReplace {
    fn std_handle(&mut self, _stream: StreamKind) -> Result<usize, u32> {
        Err(ACCESS_DENIED)
    }

    fn write_some(
        &mut self,
        _stream: StreamKind,
        _handle: usize,
        _bytes: &[u8],
        _cancel: &CancelFlag,
    ) -> Result<usize, u32> {
        Err(ACCESS_DENIED)
    }

    fn system_info(&mut self) -> (u32, u32) {
        self.inner.system_info()
    }

    fn virtual_alloc(&mut self, bytes: usize) -> Result<*mut u8, u32> {
        self.inner.virtual_alloc(bytes)
    }

    fn virtual_free(&mut self, base: *mut u8, size: usize) -> Result<(), u32> {
        self.inner.virtual_free(base, size)
    }

    fn file_replace_w16(&mut self, src: &[u16], dst: &[u16]) -> Result<(), u32> {
        if src != self.staging || dst != self.path {
            return Err(ACCESS_DENIED);
        }
        self.inner.file_replace_w16(src, dst)?;
        self.replaced = true;
        Ok(())
    }
}
