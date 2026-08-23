// Plan 419 Phase 1: 引用计数协议核心(copy-on-load 所有权协议)。
//
// 设计口径见 docs/plans/419-vm-lifecycle-three-tiers.md §1:
// - 所有"引用值"(堆对象 id / BigInt 句柄;Phase 2 起含字符串池索引)遵循
//   统一所有权协议:入栈 +1(copy-on-load)、POP 死亡 -1、STORE 转移、
//   RET 帧扫描释放、容器写旧值 -1 新值转移。
// - 偏差方向(安全纪律):漏 incref = 悬垂(致命);漏 decref = 泄漏(安全)。
//   → incref 必须咽喉点集中、审计穷举;decref 宁漏勿错。
// - RC 归零 → remove_heap_object(递归释放子引用,child_refs)。
// - 毒化 canary(debug):freed id 登记进 tombstones,get_heap_object 命中
//   即 panic,漏 incref 的 UAF 在测试期暴露。
//
// 计数语义:RC = "owned slot" 数(栈槽/局部槽/全局表项/容器字段/闭包 env)。
// insert_heap_object 不建 RC 条目(对象出世时尚无持有者);第一次 rc_push /
// rc_retain 建条目。rc_stats().live_heap 以 heap_objects.len() 为准。

use crate::vm::task::AutoTask;
use crate::vm::engine::AutoVM;
use crate::vm::virt_memory::VirtualRAM;
use dashmap::DashMap;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// 堆对象 id 基址(engine.rs heap_object_id_gen 起始值)。裸 i32 ≥ 此值按
/// 堆引用处理(与 TYPE_TO_STR 的 4000000 启发式同源,单一实现)。
pub const HEAP_ID_BASE: u64 = 4_000_000;

/// tombstone 保留时长(毒化 canary 窗口)。超龄条目在表满时惰性清理。
const TOMBSTONE_TTL: Duration = Duration::from_secs(600);
const TOMBSTONE_MAX: usize = 8192;

/// Plan 419 §1: 判定一个 NanoValue 是否为堆引用。
///
/// 三种编码并存:
/// - TAG_OBJECT(5) / TAG_LIST(6) / TAG_BIGINT(0xA):nanbox 标签编码;
/// - 裸 TAG_I32 且 payload ≥ HEAP_ID_BASE:旧式 push_i32(id) 推法
///   (native.rs/ffi 大量存在)。
/// 误报(真整数 ≥4M)只造成对不存在 id 的计数增减,rc 表按需建条目、
/// release 对无条目 id 直接跳过,净效果为零(见 plan §1 偏差纪律)。
#[inline(always)]
pub fn is_heap_ref_nv(nv: auto_val::NanoValue) -> bool {
    if auto_val::is_object(nv) || auto_val::is_list(nv) || auto_val::is_bigint(nv) {
        return true;
    }
    if auto_val::is_i32(nv) {
        return (auto_val::decode_i32(nv) as i64) >= HEAP_ID_BASE as i64;
    }
    false
}

/// Plan 419 §1: 提取堆引用的目标 id(非引用返回 None)。
#[inline(always)]
pub fn heap_ref_id(nv: auto_val::NanoValue) -> Option<u64> {
    if auto_val::is_object(nv) {
        Some(auto_val::decode_object(nv) as u64)
    } else if auto_val::is_list(nv) {
        Some(auto_val::decode_list(nv) as u64)
    } else if auto_val::is_bigint(nv) {
        Some(auto_val::decode_bigint_handle(nv) as u64)
    } else if auto_val::is_i32(nv) {
        let id = auto_val::decode_i32(nv) as i64;
        if id >= HEAP_ID_BASE as i64 { Some(id as u64) } else { None }
    } else {
        None
    }
}

// ============================================================================
// Plan 419 §9(复活):ash-gui UAF 三点诊断埋点。
//
// env P419_UAF_TRACE 控制(OnceLock 读一次,关闭时零开销):
//   P419_UAF_TRACE=              → 区间 [4M, 4M+1500](§9.4 ① 缺省窗口)
//   P419_UAF_TRACE=4000111       → 单 id 精确追踪(§9.4 ③ 收窄用)
//   P419_UAF_TRACE=4000100-4000120 → 自定区间
// 开启后对区间内 id 记录 ALLOC / FREE / retain / release / ACCESS 全事件;
// FREE 与 retain-after-free(复活竞态烟雾枪)附强制栈。ACCESS 点打在
// GET_GENERIC_FIELD 的启发式探测处,tag 来源(TAG_OBJECT vs TAG_I32≥4M)
// 直接判 H0(启发式误判)与 H1/H2(真悬垂)。
// ============================================================================
static P419_UAF_CFG: OnceLock<(bool, u64, u64)> = OnceLock::new();

fn p419_uaf_cfg() -> &'static (bool, u64, u64) {
    P419_UAF_CFG.get_or_init(|| {
        let Some(spec) = std::env::var_os("P419_UAF_TRACE") else {
            return (false, 0, 0);
        };
        let spec = spec.to_string_lossy().trim().to_string();
        if spec.is_empty() {
            return (true, HEAP_ID_BASE, HEAP_ID_BASE + 1500);
        }
        let (lo, hi) = match spec.split_once('-') {
            Some((lo, hi)) => (
                lo.trim().parse().unwrap_or(HEAP_ID_BASE),
                hi.trim().parse().unwrap_or(HEAP_ID_BASE + 1500),
            ),
            None => {
                let v = spec.parse().unwrap_or(HEAP_ID_BASE);
                (v, v)
            }
        };
        (true, lo, hi.max(lo))
    })
}

/// id 是否落在诊断区间(含总开关判定)。
pub fn p419_uaf_traces(id: u64) -> bool {
    let &(on, lo, hi) = p419_uaf_cfg();
    on && id >= lo && id <= hi
}

/// 区间是否足够窄(≤16)——窄区间才给 ALLOC 附分配栈,宽区间免启动期栈洪泛
/// (宽窗口只服务「谁被释放了」;分配站归属用单 id 收窄后补拍)。
pub fn p419_uaf_narrow() -> bool {
    let &(on, lo, hi) = p419_uaf_cfg();
    on && hi - lo <= 16
}

// ============================================================================
// Plan 419 Phase 2: 字符串池引用计数。
//
// - 仅 TAG_STRING nanbox 值参与池计数;ListData<i32> 裸负数等旧编码不参与
//   (负数与字符串 tag 在裸 i32 域不可区分 —— 计数会误伤真负数)。
// - flash 常量(load_strings 载入)标记 pinned:永活永复用、免计费 ——
//   字节码立即数只引用 pinned 条目,免重映射的关键不变量(plan §3.1)。
// - RC 归零的运行期条目 → 墓碑(内容清空释放字节)+ slot 进 freelist;
//   add_string 新内容优先复用 freelist。释放时同步删除 dedup 键
//   (freelist 复用不走 dedup,避免"一键两槽"竞态腐化)。
// ============================================================================

/// Plan 419 Phase 2: 池条目的并行状态(rc / pinned / freelist)。
/// 索引永不移动(不做物理压缩),Vec 只增不缩。
pub struct PoolState {
    pub rc: Vec<AtomicU32>,
    pub pinned: Vec<bool>,
    /// 显式墓碑标记(与 rc==0 区分:条目"创建未推栈"时 rc 也是 0)。
    pub tombstone: Vec<bool>,
    pub freelist: Vec<usize>,
}

impl PoolState {
    pub fn new() -> Self {
        Self { rc: Vec::new(), pinned: Vec::new(), tombstone: Vec::new(), freelist: Vec::new() }
    }

    pub fn ensure_len(&mut self, n: usize) {
        while self.rc.len() < n {
            self.rc.push(AtomicU32::new(0));
            self.pinned.push(false);
            self.tombstone.push(false);
        }
    }
}

/// Plan 419 Phase 2: TAG_STRING 的池索引(其他编码 None)。
#[inline(always)]
pub fn pool_idx_nv(nv: auto_val::NanoValue) -> Option<u32> {
    if auto_val::is_string(nv) {
        Some(auto_val::decode_string(nv))
    } else {
        None
    }
}

/// 测试断言钩子(plan §1 咽喉函数配套)。确定性计数,禁用 RSS 断言。
pub struct RcStats {
    /// heap_objects 表中存活的对象数。
    pub live_heap: usize,
    /// 字符串池条目数(Phase 1 为池长度;Phase 2 起为非墓碑条目数)。
    pub live_pool: usize,
    pub created_total: u64,
    pub freed_total: u64,
    /// retain+release 总流量(Phase 3 RC 消除优化的对照基线)。
    pub rc_traffic: u64,
}

impl AutoVM {
    // ====================================================================
    // Plan 419 §1 咽喉函数:引用值入栈 / 计数操作。所有"引用值入栈"点
    // 必须走这里(§2.3 审计清单);不得绕过直接 push_nv/push_i32。
    // ====================================================================

    /// 引用值入栈(+1)。非引用值直接入栈,零开销路径只有一次 tag 判定。
    /// Phase 2 起 TAG_STRING 同走池计数(pinned 条目免计费)。
    #[inline(always)]
    pub fn rc_push(&self, task: &mut AutoTask, nv: auto_val::NanoValue) {
        if let Some(id) = heap_ref_id(nv) {
            self.rc_retain_id(id);
        } else if let Some(idx) = pool_idx_nv(nv) {
            self.pool_retain(idx as usize);
        }
        task.ram.push_nv(nv);
    }

    /// Plan 419 Phase 2: 池索引入栈(+1;pinned 免计费)。
    #[inline(always)]
    pub fn rc_push_str_idx(&self, task: &mut AutoTask, idx: usize) {
        self.pool_retain(idx);
        task.ram.push_str_idx(idx as u32);
    }

    /// 裸堆 id 入栈(+1)——旧式 push_i32(id) 推法的咽喉替代。
    #[inline(always)]
    pub fn rc_push_id(&self, task: &mut AutoTask, id: u64) {
        self.rc_retain_id(id);
        task.ram.push_i32(id as i32);
    }

    /// 计数 +1(容器字段/全局表项获得持有时)。
    pub fn rc_retain_id(&self, id: u64) {
        if id < HEAP_ID_BASE {
            return;
        }
        if p419_uaf_traces(id) {
            let old = self.rc_count(id);
            // 已 tombstone 又被 retain = 复活竞态(§9.2 重入边界/TOCTOU 烟雾枪)。
            // tombstones 表只在 debug_assertions 下存在(字段级 cfg);release
            // 下 RETAIN-AFTER-FREE 探测退化为普通 retain 日志(否则 E0609)。
            #[cfg(debug_assertions)]
            let resurrected = self.tombstones.get(&id).is_some();
            #[cfg(not(debug_assertions))]
            let resurrected = false;
            if resurrected {
                eprintln!(
                    "[P419UAF] !! RETAIN-AFTER-FREE id={} (rc {} -> {}) — site:\n{}",
                    id, old, old + 1,
                    std::backtrace::Backtrace::force_capture()
                );
            } else {
                eprintln!("[P419UAF] retain id={} (rc {} -> {})", id, old, old + 1);
                // 窄区间:首次获取(0->1)附栈,识别唯一真实持有者的来历。
                if old == 0 && p419_uaf_narrow() {
                    eprintln!(
                        "[P419UAF] first-acquire site:\n{}",
                        std::backtrace::Backtrace::force_capture()
                    );
                }
            }
        }
        if std::env::var("P419_TRACE").is_ok() { eprintln!("[P419] retain {} -> {}", id, self.rc_count(id) + 1); }
        self.rc_traffic.fetch_add(1, Ordering::Relaxed);
        match self.heap_rc.entry(id) {
            dashmap::mapref::entry::Entry::Occupied(mut e) => {
                e.get_mut().fetch_add(1, Ordering::Relaxed);
            }
            dashmap::mapref::entry::Entry::Vacant(e) => {
                e.insert(AtomicU32::new(1));
            }
        }
    }

    /// NanoValue 形态的 retain(堆 + 池)。
    #[inline(always)]
    pub fn rc_retain(&self, nv: auto_val::NanoValue) {
        if let Some(id) = heap_ref_id(nv) {
            self.rc_retain_id(id);
        } else if let Some(idx) = pool_idx_nv(nv) {
            self.pool_retain(idx as usize);
        }
    }

    /// 计数 -1;归零则释放堆对象(remove + tombstone + 递归释放子引用)。
    ///
    /// 无条目时跳过(从未 retain 过的容器内部引用 / 已释放 id 的重复
    /// release)——漏 decref 是泄漏(安全方向),不 panic。
    pub fn rc_release_id(&self, id: u64) {
        if id < HEAP_ID_BASE {
            return;
        }
        if p419_uaf_traces(id) {
            let old = self.rc_count(id);
            eprintln!("[P419UAF] release id={} (rc {} -> {})", id, old, old.saturating_sub(1));
        }
        if std::env::var("P419_TRACE").is_ok() { eprintln!("[P419] release {} -> {}", id, self.rc_count(id).saturating_sub(1)); }
        self.rc_traffic.fetch_add(1, Ordering::Relaxed);
        let zeroed = {
            let Some(mut e) = self.heap_rc.get_mut(&id) else {
                return;
            };
            e.fetch_sub(1, Ordering::AcqRel) == 1
        };
        if zeroed {
            // remove_if 而非 remove:窗口内若有新 retain 把计数拉回 >0
            //(audit 缺口),保留条目让 canary/泄漏追踪可见。
            self.heap_rc.remove_if(&id, |_, a| a.load(Ordering::Acquire) == 0);
            self.free_heap_id(id);
        }
    }

    /// NanoValue 形态的 release(堆 + 池)。
    #[inline(always)]
    pub fn rc_release(&self, nv: auto_val::NanoValue) {
        if let Some(id) = heap_ref_id(nv) {
            self.rc_release_id(id);
        } else if let Some(idx) = pool_idx_nv(nv) {
            self.pool_release(idx as usize);
        }
    }

    /// 释放栈区间 [from, to) 内每个引用槽(RET 帧扫描 / 错误展开 / 任务
    /// 收尾 / native 死区结算)。不移动 sp——调用方负责。
    /// 释放后的槽位写 0:同一槽位不会被二次释放(嵌套执行里 POP 已减过
    /// 的槽,外层再扫时看到 0 即跳过)。
    pub fn rc_release_slot_range(&self, ram: &mut VirtualRAM, from: usize, to: usize) {
        // 注意不夹到 ram.sp:CALL_NAT 死区结算覆盖 sp 之上的已弹槽位;
        // 各调用方自传合理区间,这里只做越界保护。
        let to = to.min(ram.raw_nv.len());
        for i in from..to {
            let nv = ram.raw_nv[i];
            if is_heap_ref_nv(nv) || auto_val::is_string(nv) {
                self.rc_release(nv);
                ram.raw_nv[i] = 0;
            }
        }
    }

    /// 任务收尾:释放整栈 [0, sp) + actor state_vars(槽位置零)。
    /// 在结果提取完成后 / 模块 init 任务移除前调用。JOIN 语义依赖栈上
    /// 结果,故只应在"不再有人读该任务栈"时调用。
    pub fn rc_release_task_stack(&self, task: &mut AutoTask) {
        let sp = task.ram.sp;
        self.rc_release_slot_range(&mut task.ram, 0, sp);
        task.ram.sp = 0;
        for i in 0..task.state_vars.len() {
            let nv = task.state_vars[i];
            if is_heap_ref_nv(nv) || auto_val::is_string(nv) {
                self.rc_release(nv);
                task.state_vars[i] = 0;
            }
        }
    }

    /// 查询 id 的当前计数(无条目 = 0)。
    pub fn rc_count(&self, id: u64) -> u32 {
        self.heap_rc
            .get(&id)
            .map(|a| a.load(Ordering::Acquire))
            .unwrap_or(0)
    }

    /// 测试断言钩子。
    pub fn rc_stats(&self) -> RcStats {
        RcStats {
            live_heap: self.heap_objects.len(),
            live_pool: self.pool_live_count(),
            created_total: self.rc_created_total.load(Ordering::Relaxed),
            freed_total: self.rc_freed_total.load(Ordering::Relaxed),
            rc_traffic: self.rc_traffic.load(Ordering::Relaxed),
        }
    }

    /// RC 归零的真释放:摘表 → 墓碑 → 递归释放子引用。
    fn free_heap_id(&self, id: u64) {
        let Some(arc) = self.remove_heap_object(id) else {
            return;
        };
        if p419_uaf_traces(id) {
            let tag = arc
                .read()
                .map(|g| format!("{:?}", g.type_tag()))
                .unwrap_or_else(|_| "<locked>".to_string());
            // §9.4 ① free-site 栈:直接回答「谁释放了它」。
            eprintln!(
                "[P419UAF] FREE id={} type={} — free site:\n{}",
                id, tag,
                std::backtrace::Backtrace::force_capture()
            );
        }
        self.rc_freed_total.fetch_add(1, Ordering::Relaxed);
        #[cfg(debug_assertions)]
        {
            // Plan 419: 摊销清理 —— 每 4096 次插入才做一次 TTL 全扫(每 freeing
            // 都扫会在 churn 场景退化成 O(n²);canary 窗口相应放宽)。
            self.tombstones.insert(id, Instant::now());
            if self.tombstones.len() > TOMBSTONE_MAX
                && self.rc_freed_total.load(Ordering::Relaxed) % 4096 == 0
            {
                self.tombstones.retain(|_, t| t.elapsed() < TOMBSTONE_TTL);
            }
        }
        // 递归释放:子引用的 stakes 随父对象死亡。
        let children = {
            let guard = arc.read().unwrap();
            guard.child_refs()
        };
        for child in children {
            self.rc_release_id(child);
        }
    }

    // ====================================================================
    // Plan 419 Phase 2: 池计数咽喉
    // ====================================================================

    /// 池条目 +1(pinned 免计费)。越界索引(旧编码负数误入)跳过。
    pub fn pool_retain(&self, idx: usize) {
        let st = self.pool_state.read().unwrap();
        if idx >= st.rc.len() || st.pinned[idx] {
            return;
        }
        let cur = st.rc[idx].fetch_add(1, Ordering::Relaxed);
        // Plan 423 P5 续修(诊断设施):指定索引生死链 trace。
        if crate::pool_trace_idx() == Some(idx) {
            eprintln!("[P419POOL] retain {} (rc {} -> {})", idx, cur, cur + 1);
        }
        let _ = cur;
        self.rc_traffic.fetch_add(1, Ordering::Relaxed);
    }

    /// 池条目 -1;归零 → 墓碑(清内容)+ freelist + 删 dedup 键。
    pub fn pool_release(&self, idx: usize) {
        let zeroed = {
            let st = self.pool_state.read().unwrap();
            if idx >= st.rc.len() || st.pinned[idx] {
                return;
            }
            let cur = st.rc[idx].fetch_sub(1, Ordering::AcqRel);
            if crate::pool_trace_idx() == Some(idx) {
                eprintln!("[P419POOL] release {} (rc {} -> {})", idx, cur, cur - 1);
            }
            cur == 1
        };
        self.rc_traffic.fetch_add(1, Ordering::Relaxed);
        if zeroed {
            self.pool_free_idx(idx);
        }
    }

    /// 归零真释放:置墓碑、清内容、进 freelist、删 dedup 键(一键一槽不变量)。
    fn pool_free_idx(&self, idx: usize) {
        if crate::pool_trace_idx() == Some(idx) {
            eprintln!("[P419POOL] FREE {} (tombstone + freelist)", idx);
        }
        let key = {
            let mut strings = self.strings.write().unwrap();
            let key = strings.get(idx).cloned();
            if let Some(k) = &key {
                strings[idx] = Vec::new();
            }
            key
        };
        if let Some(k) = key {
            self.string_dedup.lock().unwrap().remove(&k);
        }
        {
            let mut pool = self.pool_state.write().unwrap();
            pool.tombstone[idx] = true;
            pool.freelist.push(idx);
        }
        self.rc_freed_total.fetch_add(1, Ordering::Relaxed);
    }

    /// 池墓碑查询(canary 用)。
    pub fn pool_is_tombstone(&self, idx: usize) -> bool {
        let st = self.pool_state.read().unwrap();
        idx < st.tombstone.len() && st.tombstone[idx]
    }

    /// live 池条目数(总长 - freelist)。
    pub fn pool_live_count(&self) -> usize {
        let strings_len = self.strings.read().map(|s| s.len()).unwrap_or(0);
        let free = self.pool_state.read().unwrap().freelist.len();
        strings_len.saturating_sub(free)
    }

    /// 池条目当前计数(pinned 视作永久;无记录 = 0)。
    pub fn pool_count(&self, idx: usize) -> u32 {
        let st = self.pool_state.read().unwrap();
        if idx < st.rc.len() {
            if st.pinned[idx] { u32::MAX } else { st.rc[idx].load(Ordering::Acquire) }
        } else {
            0
        }
    }

    /// 毒化 canary 检查(debug):命中已释放 id 即 UAF。
    #[cfg(debug_assertions)]
    pub fn rc_check_tombstone(&self, id: u64) {
        if let Some(t) = self.tombstones.get(&id) {
            let age = t.elapsed();
            drop(t);
            panic!(
                "[RC canary] use-after-free: heap object {} was freed {:.1}s ago",
                id,
                age.as_secs_f32()
            );
        }
    }
}
