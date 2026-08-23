// backend_abi.rs — Plan 061:外部后端 cdylib 插件 ABI(merged 模式)。
//
// 设计(designs/ash-gui-external-backend.md §3):后端项目(如 ash-server)
// 整体编译为 cdylib,导出两个符号;宿主(`auto run` merged 编排)经
// libloading 装载、校验 ABI 版本后调注册入口,把 api.at 端点实现注册进
// 既有 vm::host_bridge 注册表(与 M3 手写宿主 ash-runner 同一张表),
// 事件回流复用 renderer::inject_shell_event(SSE 同格式)。
//
// 约束:符号用 `extern "Rust"`(同工具链同机构建——后端与宿主出自同一
// target 树,天然满足;跨工具链装载由 ABI 版本号拒载兜底)。

use std::path::Path;
use std::sync::Arc;

/// 宿主与后端约定的 ABI 版本。不匹配即拒载(要求重建后端)。
pub const BACKEND_ABI_VERSION: u32 = 1;

/// api.at 端点实现签名 —— 与 vm::host_bridge::HostCallFn 同构
/// (args JSON 字符串入,结果 JSON 字符串出)。
pub type BackendHostCallFn = Arc<dyn Fn(&str) -> Result<String, String> + Send + Sync>;

/// 宿主提供给后端 cdylib 的注册回调表。
pub trait BackendRegistry: Send + Sync {
    /// 注册一个 api.at 端点实现(名字 = #[api] 裸函数名)。
    fn host_call(&self, name: &str, f: BackendHostCallFn);
    /// 事件回流(SSE 同格式 JSON;tag = payload["event"])。
    /// 返回 false = 宿主侧通道不可用(后端可自行决定丢弃/记日志)。
    fn inject_event(&self, tag: &str, json: &str) -> bool;
    /// 后端日志通道(宿主决定路由到 stderr/日志文件)。
    fn log(&self, msg: &str);
}

/// cdylib 须导出的版本符号类型。
pub type AbiVersionFn = unsafe extern "Rust" fn() -> u32;
/// cdylib 须导出的注册符号类型。
/// Arc 传递:后端需把 registry 挪进自己的事件泵线程(跨线程共享),
/// &dyn 不可跨线程 —— 宿主侧以 Arc 交割。
pub type RegisterFn =
    unsafe extern "Rust" fn(std::sync::Arc<dyn BackendRegistry>) -> Result<(), String>;

/// 装载成功的后端句柄。**必须保持存活**到进程退出/前端运行结束——
/// 卸载库会令已注册闭包的 vtable 悬垂。
pub struct LoadedBackend {
    #[allow(dead_code)]
    lib: libloading::Library,
}

/// 装载并注册外部后端 cdylib。
pub fn load_backend_cdylib(
    path: &Path,
    reg: std::sync::Arc<dyn BackendRegistry>,
) -> Result<LoadedBackend, String> {
    unsafe {
        let lib = libloading::Library::new(path)
            .map_err(|e| format!("failed to load backend library {}: {e}", path.display()))?;
        let ver: libloading::Symbol<AbiVersionFn> = lib
            .get(b"auto_backend_abi_version")
            .map_err(|_| "missing `auto_backend_abi_version` export (not an Auto backend cdylib?)".to_string())?;
        let v = ver();
        if v != BACKEND_ABI_VERSION {
            return Err(format!(
                "backend ABI version mismatch: backend={v}, host={BACKEND_ABI_VERSION} — rebuild the backend project"
            ));
        }
        let register: libloading::Symbol<RegisterFn> = lib
            .get(b"auto_backend_register")
            .map_err(|_| "missing `auto_backend_register` export".to_string())?;
        register(reg).map_err(|e| format!("backend registration failed: {e}"))?;
        Ok(LoadedBackend { lib })
    }
}
