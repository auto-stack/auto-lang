//! PLAN-047（档 C SD-08）: 依赖录制核心类型。
//!
//! 本模块**不挂 feature 门**——`AutoVM` 录制槽与读臂挂钩（vm/engine.rs）
//! 与 UI 侧 memo 门宿主（ui/memo_deps.rs，ui-interpreter 门控）共用同一组
//! 类型；vm 模块不得依赖 ui 门控面，故类型单源落此处，memo_deps 转发导出。

/// 依赖键 path 约定：`"*"` = 该堆对象的任意内容读（容器粗粒度保守面——
/// 原地突变归因不可达字段级时的正确性兜底；字段级归因见 SD-09 per-path
/// 版本表 [`crate::vm::engine::AutoVM::bump_path`]）。
pub const DEP_PATH_ANY: &str = "*";

/// 依赖键 = (堆对象 id, path)。具名字段读记字段名；容器/结构体整体展开
/// 记 [`DEP_PATH_ANY`]。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DepKey {
    pub heap_id: u64,
    pub path: String,
}

impl DepKey {
    pub fn field(heap_id: u64, path: &str) -> Self {
        Self {
            heap_id,
            path: path.to_string(),
        }
    }

    pub fn any(heap_id: u64) -> Self {
        Self {
            heap_id,
            path: DEP_PATH_ANY.to_string(),
        }
    }
}

/// 单次录制预算：超限 → `overflow` 置位、集清空（条目弃动态 dep 集落回
/// 静态扫描路径——宁缺勿错，正确性下限只允许变慢）。
pub const REC_DEP_BUDGET: usize = 256;

/// 录制状态（guard 激活期持有；guard 嵌套 = 外层收编内层并集——外层
/// 条目因此覆盖内层 computed/子求值的全部依赖）。
#[derive(Debug, Default, Clone)]
pub struct RecState {
    pub deps: std::collections::BTreeSet<DepKey>,
    pub overflow: bool,
}

impl RecState {
    /// 录一条依赖。预算超限 → 弃整个集（半录制集 = 盲区，绝不半信）。
    pub fn record(&mut self, key: DepKey) {
        if self.overflow {
            return;
        }
        if self.deps.len() >= REC_DEP_BUDGET {
            self.overflow = true;
            self.deps.clear();
            return;
        }
        self.deps.insert(key);
    }

    /// 内层集收编进外层（并集 + overflow 传播）。
    pub fn absorb(&mut self, inner: &RecState) {
        self.overflow |= inner.overflow;
        if self.overflow {
            self.deps.clear();
            return;
        }
        for k in &inner.deps {
            if self.deps.len() >= REC_DEP_BUDGET {
                self.overflow = true;
                self.deps.clear();
                return;
            }
            self.deps.insert(k.clone());
        }
    }
}
