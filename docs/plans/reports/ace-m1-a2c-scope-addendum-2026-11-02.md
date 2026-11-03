# ACE M1范围增补：A2C第二目标（2026-11-02）

> **2026-11-03后续M1裁定**：[架构机制完整性增补](ace-m1-architecture-completeness-decision-2026-11-03.md)在本文件C新增门之外加入SYN-01..06及ARCH-01..08。CA-E最终收口须合并新门，原门+C门不能单独签新M1；统一语法/机制审计尚待正式立项，不能自动扩写925/926或冒领已有实施证据。

状态：用户已批准新增M1目标；本文件冻结目标和边界，实施/测试/认证尚未执行。独立于2026-10-28决策件及其2026-11-01 §13，不改该文件或既有批准SHA。

## 1. 授权与取代范围

本轮用户明确："我们把A2C目标也加入到M1的目标中去"，并要求只做设计/规划、核A2X与797基线、明确一致性/FFI和913–917边界。

- A2C成为Rust版ACE M1的必需第二转译目标，不再是可选后继或仅标量演示。
- 取代Design51/共享A2X路线中"第二目标优先Python"的排期建议，以及上一轮"A2C不加入M1前置"的建议。Python仍是后续异构目标，未承诺本次M1支持。
- 继承所有已批准NA/accepted-diff与A2R双轴、原313 required及其独立关闭条件；不扩大这些规则到新C目标，不重开已归档Plan。
- 只新增设计和路线，不修改production、current Specs、原source/expected、binding.tsv/applicability.json/source-manifest.json、913–917实施合同或进度；本轮不取Plan号、不建树/材料、不编译/测试/认证、不启动会话或agent。

设计主体：[Design51 §13](../../design/51-shared-a2x-and-hir-to-rust.md#13-a2c第二目标与m1范围增补2026-11-02target-state)；执行路线：[A2C路线](../../design/strategy/shared-a2x-and-hir-to-c-roadmap.md)。

## 2. M1新增必需范围

| 必需面 | 关闭要求 | 明确边界 |
|---|---|---|
| C核心与共享验证 | Checked HIR→共享SemanticView/FunctionPlan/依赖→独立C provider→完整文件集；core/scalar、名义record/text/text-record、有限List/Variant各域原源真闭环 | 以892/902/904已交付支持面为C基线，不能只add函数或只语法打印；新域不冒完整Auto语言 |
| 语义与资源 | 源码序/单次求值/短路、checked i32、Unicode/NUL、view/mut/move/clone、失败原子/逐owner清理；C/Rust各对独立oracle | C语法/默认算术/求值与内存规则不能替Auto语义；旧A2R/旧A2C不自动授信新目标 |
| 增量/恢复/预算 | 首例文件化、fresh/hit/restart/body-edit、未改函数真实生产/工件IO0及同进程对象代际；恢复深验/故障后修复再恢复；P90≤2000ms与原资源/规模门 | 范围及支持编辑集在正式Plan逐条冻结，不能以已知冷路径当暖增量PASS；C编译器可能重编更大单元，另行计量 |
| 有限ABI桥 | A2C生成函数接明确C/AutoABI wrapper，与独立C双方真实双向互调；沿797既有Windows x64候选profile/准入/原方释放/同步per-call callback | 不要求完整C header importer、Rust crate/Python桥、跨线程回调、动态loader或跨平台通用FFI；受限typed外部声明不冒任意Auto源码FFI语法 |
| 四端一致性与认证 | AC/AV/A2R-HIR/A2C-HIR在冻结共同源域分别对独立oracle；C实际文件集和手写reference均真实编译运行，证书/依赖/工具身份严格准入 | 原旧A2R日常金样义务仍在；不得四端相等就忽略公共语义bug，也不要求四端拥有相同HMR/暂停机制 |

不由本增补自动加入：C闭包/HOF、Dyn、Unit/console全入口、宽源码算术、用户泛型/模块/完整stdlib、多平台、旧生成器默认切换/删除、完整AJ/OSR。它们保M2/M3/M4能力队列；C端若正式认领新能力，另冻结输入/判据与重新进入条件，不能事后缩共同子集凑绿。

## 3. 新M1签章与分母

新目标谓词为：

`M1 = M1-base-908 ∧ C-core-resource ∧ C-incremental-recovery ∧ C-bounded-ABI ∧ Q4-semantic-evidence ∧ expanded-registration-valid`。

M1-base-908保留现行四路径524格/313 required/45 future与38 cert-gap独立账及原报告身份。旧acceptance/2的can_claim_m1只证明该历史基面，不能单独授权本轮扩大后的M1签章；新增C门未实现/未执行期间，总M1状态必须为pending/不可签。

新增目标采用独立版本化target ledger和组合报告（建议ace.m1.acceptance/3，由CA-E正式Plan冻结），不能直接在12列binding插第五列破坏在途工具。新ledger在CA-A先冻结初稿、各包逐项落实，CA-E收口；每行包含稳定source/case/hash、语言profile、实际目标ID、适用与批准来源、oracle、证据/工具/依赖身份、状态/owner，列出缺失/未执行/infra，不得隐藏未知目标或零选择。

目标ID明确区分AC-native、AV、ACE-A2R-HIR、ACE-A2C-HIR、legacy-AVM、legacy-A2R、legacy-A2C。四端一致性Q4用前四者；后三者是既有义务/参考，不能替代新producer。不是机械要求131×5或全部示例全后端跨积；C新增必需集须从本节已批准域及892/902/904全部冻结语料逐项映射，C缺项仍gap，不能用动态求交集把它排除。

## 4. 拆包与交接决定

首期预计五个适中实施Plan，暂用CA-A..E工作标签，未取正式号：

1. CA-A：共享目标合同与session/store去Rust耦合，带C标量真实消费者和首例/认证/增量。
2. CA-B：C record/text、模式/资源/已有static/view方法及清理。
3. CA-C：C集合/变体、拥有元素、活动臂/失败清理与增量。
4. CA-D：797 profile下的有限C/AutoABI wrapper与源/typed分层互调。
5. CA-E：新增M1 target ledger/四端证据/总签章门；规则/数据设计从CA-A并行开始，最终跨包独立验收。

顺序CA-A→CA-B→CA-C；CA-D消费CA-A正式接缝，所需资源形态按CA-B稳定接口分阶段；CA-E不承接其他包遗漏的必需测试，每包首例即纳一致性/独立认证。未知前端事实或ABI语义若需大扩面，先needs_replan，不先拿五个号码占位。

913恢复、914场景观察、916旧A2R已实际在途（916头部滞后不代改）；915/917仍drafting。新C私有provider/参考设计可并行，共用HIR/资源事实/ManagedStore、A2X注册/worker/SDK/测试/证书/登记按实际符号单写；不消费他方WIP、不新增D913..917整波前置。具体窗口和case见A2C路线。

## 5. 本轮边界与后续沉淀

正式实施Plan分别提出auto-a2x/auto-a2c/auto-abi/auto-test-case的Spec delta；本轮不把目标态写成current已实现。既有批准件SHA=874ccc1198bf23c284fa588a6dfcdee80eecc8f5a0e392b88b34ddd74db0875b；binding=67bf520d26e7d541f8fd5f44b8c218da9d40b67d3fd808b4b3e801b89ec47fb9；applicability=74c9109bbe7ec8596e6fa1bbc2ab9f4f5640725a748d0e63e53eb5e07ee275c0；source-manifest=34d8512b43b11e23dc551e6220288d8748a5d997b337da11171014bc17ca1fca。本增补须以独立来源hash由新ledger纳入，不回写旧批准链冒原来已批准C。
