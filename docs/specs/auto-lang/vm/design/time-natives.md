# time 族 VM 运行时 shim 契约（now_ms / now_sec / now / Instant.elapsed）

> PLAN-701 供④（2026-09-25）。catalog 1200/1201/1205 自 Plan 396 起
> 登记但长期无运行时 shim（裸/use 两形态均返 0，m1-supply §9 登记）——
> 本件实装并把三方语义钉死（GOAL-003）。

## 三方一致（GOAL-003）

`stdlib/auto/time.at`（声明+文档注释）/ `stdlib/auto/time.rs.at`
（#[rs] 实现）/ `crates/a2r-std/src/time.rs`（a2r 轨）三方同为
**Unix epoch 钟值**：

| 端点 | id | 返回 | 语义 |
|---|---|---|---|
| `auto.time.now_ms` | 1200 | i64 | epoch 毫秒（非单调钟） |
| `auto.time.now_sec` | 1201 | i64 | epoch 秒 |
| `auto.time.now` | 1205 | str | epoch 秒十进制串（a2r `time_now` 别名同形） |

VM shim（`vm/native.rs` shim_time_now_ms/now_sec/now）用同源
`SystemTime` 取值——对拍测试
（`tests/plan701_supply_probes.rs`）钉 VM 轨 vs `a2r_std::time` 差值
<5s。**计划正文「单调钟」措辞为立项笔误**：.at 声明与 a2r-std 实现
均为 epoch，三方一致优先（执行期勘定，R-1 口径）。

## Instant.elapsed() 返回约定（T-06 勘定）

- **旧形**：shim 推「Nms」字符串 + null 双推、catalog 名表 Void——
  .at 侧实得 None（m1-supply §9 观察件）。
- **现约定**：`Instant.elapsed() -> i64` 毫秒 int（单调钟；catalog
  名表同步 Void→I64）。`Instant::now()`（1203）维持 opaque 句柄形。
- **跨轨边界**：a2r/trans 轨的 `.elapsed()` 保持真实 Rust `Duration`
  （opaque 不透明形）——VM 轨为毫秒 int。不透明类型跨轨既有边界类，
  v1 不对齐（.at 侧无 Duration 可言语义，int 形为可用性优先）。

## 桥面与算术边界（执行期勘定增量）

- `nv_to_pub_value`（vm_bridge）补 TAG_I64 臂（此前落 decode_i32 兜
  底=桥面垃圾值；Plan 522 T4 f64 臂同族缺口）——宿主侧现可拿全宽
  i64 返回值。
- **.at 层 TAG_I64 算术不可用**（DIV 解码错位实测：`now_ms()/1000`
  → 0）——预存平台缺口，不在本件清偿面；unified print 的 TAG_I64 臂
  在档，BENCH 标记行可直出毫秒值（print 形/字符串透传形不涉 .at 内
  算术）。下游 bench 差值形消费前置条件：差值在 .at 内计算需等平台
  i64 算术面另件，或改双读出-host 相减形。

## 消费位

auto-edit PLAN-014 T-07：bench.py BENCH 标记升 app 侧毫秒值（JSONL
字段改源，host 时间戳保留对照列）。
