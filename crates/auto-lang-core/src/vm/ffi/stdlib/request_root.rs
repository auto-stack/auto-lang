//! PLAN-884 T-04：请求文件根（TLS + RAII guard + 解析）自 stdlib 壳迁入。
use super::*;

thread_local! {
    static REQUEST_FILE_ROOT: std::cell::RefCell<Option<StdPathBuf>> = const { std::cell::RefCell::new(None) };
}

/// Scoped relative-file root for hosted project execution. This never changes
/// the process working directory and is restored when the execution ends.
pub struct RequestFileRootGuard {
    previous: Option<StdPathBuf>,
}

impl RequestFileRootGuard {
    pub fn for_source_path(path: Option<&str>) -> Self {
        let root = path
            .and_then(|path| Path::new(path).parent())
            .map(Path::to_path_buf);
        let previous = REQUEST_FILE_ROOT.with(|slot| slot.replace(root));
        Self { previous }
    }
}

impl Drop for RequestFileRootGuard {
    fn drop(&mut self) {
        let previous = self.previous.take();
        REQUEST_FILE_ROOT.with(|slot| {
            slot.replace(previous);
        });
    }
}

pub(crate) fn resolve_request_file_path(path: &str) -> Result<StdPathBuf, String> {
    let root = REQUEST_FILE_ROOT.with(|slot| slot.borrow().clone());
    let Some(root) = root else {
        return Ok(StdPathBuf::from(path));
    };
    let relative = Path::new(path);
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err("request file paths must be relative to the project root".to_string());
    }
    Ok(root.join(relative))
}
