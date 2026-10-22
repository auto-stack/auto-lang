# PLAN-815 T-07 完整验收 · 子批 1（2026-10-22）

code `661c0573c0e16630cd9c2c8e8da91c7736c40afd`（仅测试/fixture +151/-6；
生产代码与二进制零变化——bin SHA256 `f21db181…` 不变）。

| 待办项 | 本批交付 | 证据 |
|---|---|---|
| 协议 fixture 四种服务端回执 | `consumer_server_receipts_round_trips`：DescribeOk（真实 ToolDescriptor+LifecyclePolicy::contract）/ValidateOk（ValidateProjectReport 含 Error Finding+RequestSummary）/GenerateOk（GenerationReport 多 frontends）/BuildOk（BuildReport failed 具名 error）wire 往返 | fixture 7/7（原 5/5） |
| 帧深度/超深负控 | `consumer_frame_depth_boundaries`：恰在 `MAX_FRAME_JSON_DEPTH`(122) 的最深消息可写可读无损；深一级=具名 `Malformed "nests deeper than the transmittable envelope"`（json_depth 语义=包裹层+1，边界值按此构造） | 同上 |
| real_tree Linux 可移植缺口（已披露遗留） | 跨平台化：findstr→grep、cmd 嵌套 ping→`sh -c 'sleep 30 & exec sleep 30'`（cfg 分裂）；**Linux 首次通过** | WSL --locked：real_tree+unix_reader+captured_drain 4/4 + BoundOverrun 慢测 10.07s（1/1） |

保持：Windows real_tree 1/1、proc_host 9/9、闭包门 PASS、fixture 零告警、
fmt 全净。

## 剩余 T-07 待办（子批 2+，未做不冒称）

- 792/fresh/locked 发行矩阵：新 tool/producer/source/embed 资源、重定位、
  受管发布失败/保留/恢复、独立消费（旧 T-04 收据不覆盖新增 run/build/
  helper 变化，需 driver 扩面）。
- 资源/许可审计：资产 owner/license/源唯一性与真实发行内容互核。
- V-01..04 完整字段与兼容矩阵：mode/output_root/input_sources/dep_roots/
  theme/sidecar/外部 back 逐 profile 消费/固定限制对照。
- 777 master 并集（归 merge/T-10）。
