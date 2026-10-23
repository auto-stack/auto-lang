# Repair entry — PLAN-838 r1 / final R1

2026-10-24，原树plan-838-dev/8494e06d4056e7f4e0f6dd1925e128bb93a83ea6，主检出代码面clean；恢复单一实现写权，未独立review或merge。用户提供needs_fix，T03/06/07/08按原合同修复，无语义降约。

| finding | invariant / actual producer → consumer | sibling states and closing oracle |
|---|---|---|
| F01 | Storage Save/Save As snapshot → normalized target → root lock/entry rename；任何canonical blob/lock均不得当scene entry | Windows大小写/..及同父别名、已有/新目标、冲突原路径、pending receipt。拒绝在写和确认前发生；原blob/entry/world/dirty/path保留，普通新scene正控。 |
| F02 | raw GLB → strict admission → gltf typed readers → AGP → Auto/GPU | POSITION/NORMAL/TEXCOORD_0/indices 的component/dimensions/normalized，base/sparse/interleaved，未引用mesh。非法具名Err、decode/upload零增量、旧document/history/owner/像素保持、合法重试。 |
| F03 | whole-file morph声明 → admission | mesh/node weights及primitive targets，含空声明/未引用mesh；decoder前拒绝，不删字段静态渲染；无morph正控。 |
| F04 | raw node matrix/TRS声明 → transform selection | matrix+translation/rotation/scale，含未引用node；选择前拒绝，matrix-only/TRS-only合法且具体geometry正确/原bytes不改。 |

先原样重放review持久输入并记录基线；全部修复后加永久兄弟路径正反控、定向assets/storage/Host与native/源码consumer的受影响面，提交冻结再交独立review。根整档和完整merge矩阵保持pending。仅1次final needs_fix，无两次同因诊断触发。
