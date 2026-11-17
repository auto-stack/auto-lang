//! PLAN-971 B（T-03/V-04）：Playground 的 ACE 会话宿主接入。
//!
//! Notebook 会话不再持有旧 AutoVM REPL 会话：每个 notebook 会话就是一个
//! `ace_session`（PLAN-970 中立合同）会话，状态（累积程序/变量 carry 表/
//! 世代)由 ACE session host 单线程属主持有。两个用户的会话因此结构性隔离
//! （各自独立的会话 id 与 carry 表），关闭即 `Close` 释放，不存在跨会话
//! 共享状态，也没有旧 VM 回退路径。
//!
//! 宿主进程级安装一次（`OnceLock`）：guard 常驻进程生命周期，测试进程与
//! 产品进程同一路径，重复调用不重复安装。

use auto_lang_core::{ace_session_op, AceSessionOp, AceSessionOutcome, AceSessionReport};

/// 安装（或确认已安装）ACE 会话宿主。返回是否可用；失败时调用方必须把
/// 会话操作当作具名失败呈现，绝不再落到旧 REPL。
///
/// F-971-B-04（复审 B1）：ACE 桥 crate 仅 Windows 有实现。非 Windows 上这里
/// 不引用其符号（否则是编译期断链），而是返回 false——notebook 会话于是走
/// 既有具名 `ace.session/host-unregistered` 失败路径，与 LSP 侧的
/// `cfg(not(windows))` 处理一致。
#[cfg(windows)]
pub fn ensure_session_host() -> bool {
    static SESSION_HOST: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *SESSION_HOST.get_or_init(|| {
        match auto_lang_ace_host::session_bridge::install_session_host(
            auto_lang_core::AceSessionLimits::default(),
        ) {
            Ok(guard) => {
                // 进程生命周期持有：guard 一旦 drop 会关闭所有会话（会话
                // 属主线程停止），Playground 需要宿主与进程同寿。
                std::mem::forget(guard);
                true
            }
            Err(_) => false,
        }
    })
}

/// 非 Windows：ACE 会话宿主不可用（具名失败路径由调用方呈现）。
#[cfg(not(windows))]
pub fn ensure_session_host() -> bool {
    false
}

/// 一个 notebook 会话句柄（ACE session id + 宿主可用性）。
#[derive(Debug)]
pub struct AceNotebookSession {
    id: Option<String>,
}

impl AceNotebookSession {
    /// 新建空会话（无源种子）。
    pub fn create() -> Self {
        if !ensure_session_host() {
            return AceNotebookSession { id: None };
        }
        let report = auto_lang_core::ace_session_create(None, None);
        match report.outcome {
            AceSessionOutcome::Ok => AceNotebookSession { id: report.session },
            _ => AceNotebookSession { id: None },
        }
    }

    /// 执行一段输入（表达式/语句/项定义）。
    pub fn eval(&self, input: &str, wall: std::time::Duration) -> AceSessionReport {
        let Some(id) = self.id.as_deref() else {
            return unavailable("the ACE session host is not installed in this process");
        };
        ace_session_op(
            id,
            AceSessionOp::Eval {
                input,
                wall: Some(wall),
                cancel: None,
            },
        )
    }

    /// 当前 carry 表（变量）。
    pub fn vars(&self) -> AceSessionReport {
        let Some(id) = self.id.as_deref() else {
            return unavailable("the ACE session host is not installed in this process");
        };
        ace_session_op(id, AceSessionOp::Vars)
    }

    /// 累积源文本（用于函数清单等只读检视）。
    pub fn snapshot(&self) -> AceSessionReport {
        let Some(id) = self.id.as_deref() else {
            return unavailable("the ACE session host is not installed in this process");
        };
        ace_session_op(id, AceSessionOp::Snapshot)
    }

    /// 关闭会话并释放其 pinned 世代（幂等）。
    pub fn close(&mut self) {
        if let Some(id) = self.id.take() {
            let _ = ace_session_op(&id, AceSessionOp::Close);
        }
    }
}

fn unavailable(message: &str) -> AceSessionReport {
    // 具名宿主失败报告：绝不以空成功冒充可用会话。
    AceSessionReport {
        session: None,
        outcome: AceSessionOutcome::HostFailure {
            code: "ace.session/host-unregistered".to_string(),
            message: message.to_string(),
        },
        revision: 0,
        value: auto_lang_core::AceSessionValue::None,
        eval_id: None,
        vars: Vec::new(),
        snapshot: None,
        rebuilt: false,
        wall_ms: 0.0,
    }
}
