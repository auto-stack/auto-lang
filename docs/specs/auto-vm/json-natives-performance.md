# auto-vm / json-natives-performance（PLAN-995）

状态：delivered（随 PLAN-995 交付）· 日期：2026-11-08
消费方：`json.parse` / `json.stringify` / `json.to_value` / `json.from_value` 的全部 VM 调用方（auto-reader catalog 路径、HTTP body 解析、AutoUI 状态序列化）。

## 语义保持承诺（优先于性能）

1. **物化形状**（`json.parse`/`json.to_value`）：对象→`__json_object` GenericInstanceData（字段插入序）、数组→`ListData<Value>`；数值归一 **i64 优先**（serde 无符号语法形态的正整数亦归一 i64→i32 装箱/I64；仅真超 `i64::MAX` 的 u64 走 BigIntData/u64 宽编码）；f64→Double；str→AutoStr；子 VmRef 插入即 retain。
2. **序列化形状**（`json.stringify`/`json.from_value`）：GenericInstanceData 按字段插入序、ObjectData 按键字典序（PLAN-057 已登记的观感差）、BigIntData 按 i64/u64、非有限浮点报错（文本 `json.from_value: NaN/Inf float|double`）。
3. **深度上限 64**，错误文本前缀 `json_to_vm_value*:` / `vm_value_to_json:` 原样透传；语法错误前缀 `json.parse: invalid JSON:`。
4. **错误路径栈干净**：`parse_json_direct` 先 `IgnoredAny` 零分配全量语法校验再物化——语法错误零推送；深度类错误 unwind 部分推送（含影子份额释放）。
5. **等价 oracle（永久）**：`vm::tests_json_hazards::w02_json_roundtrip_byte_equality`——`stringify(parse(x)) == x` 逐字节（serde canonical 文本、字面 UTF-8、`\"`/`\\`/`\n`/`\t` 转义、i64 域大整数、嵌套、空容器）；`\uXXXX` 转义**非对称**（解码后按字面回发）不入 oracle。

## 基准夹具与测量口径

- 夹具（冻结，plan-995 scope）：`catalog-1000.json`（千册 216,616B，sha256 头 943a22f1）+ `deep-40.json`（深度 40/100KB 叶串，sha256 头 781a5d60）。
- 口径：service 级 .at 端点（10 次迭代/端点摊平 HTTP 噪声），**同机静窗口 ≥2-3 run band**，跨机/负载窗口数字不比较（round23 实证 ±2× 浮动）。
- 记录方式：before/after band 中位数；负控（真不等必 false）随行；测量脚本与 band JSON 归 plan scope，结论数字折叠入计划文件。

## 改善记录

| 项 | before（master b90c17dd1） | after（PLAN-995 f31b85a84） | 机制 |
|---|---|---|---|
| catalog parse ×10 | 0.194/0.197/0.233 s | 0.116/0.137/0.170 s（**中位 -30%**） | 单趟物化（serde visitor 直建 VM 堆值，消除中间 serde_json::Value 树） |
| catalog stringify ×10 | 0.149/0.149/0.156 s | 0.075/0.095/0.096 s（**中位 -36%**） | 直序化（VmJsonDirect，Serialize 直写） |
| deep parse/stringify ×10 | 0.061-0.084 / 0.039-0.045 s | band 重叠（持平） | 深夹具 97% 为单串叶，无树可省 |

## 边界与已知非瓶颈

- parse 在 .at 大文本路径**已非瓶颈**（~14ms/216KB 次）；`.at` 层重建累积（STR_CAT 链 + 逐字段 intern）主导（千册证明 ~1.8-2.4s，e2e cold 分量对账见 PLAN-995 §9 T-04）。
- 大文本优化方向变更须经本合同语义保持承诺 + 等价 oracle 全绿后记录于上表。
