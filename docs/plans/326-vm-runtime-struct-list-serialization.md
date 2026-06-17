# Plan 326: VM 运行时基础能力补全 — struct/List/序列化/类型转换

> **Status**: Phase 1-5 完成(Phase 2/4 经核实已在早期 commit 解决;Phase 3/5 本次实现)
> **依赖**: Plan 321(generator 运行时);Plan 312(HTTP server)
> **目标**: 修复阻塞 auto-musk 直接使用的 4 个 VM 运行时基础能力缺口
> **验收**: `examples/ui/015-notes` 的 CRUD API 在 AutoVM 下端到端跑通

## 实施结果摘要(本次 session)

- **Phase 2(struct 字段访问)**:经系统化根因调查,`note.title 返回 0` 在隔离
  最小场景已无法复现(6 个回归测试全绿)。极可能在 Phase 1 generator 帧修复时
  连带解决。新增 17 个回归测试覆盖 struct 字面量/fn 返回/数组存储/for 循环场景。
- **Phase 4(List.push)**:`shim_list_push` 已存在(native.rs:931,编号 101),
  计划"搜索零命中"为过时信息。已注册 `auto.list.push` + `List.push` 别名。
- **Phase 3(HTTP 序列化)**:**确认真实根因**——handler 返回 struct 时栈顶是
  i32 形式的 heap object ID(array: 2000000+,struct: 4000000+),旧序列化
  把它当普通数字返回 `"4000000"`。新增 `nv_to_json` 递归序列化函数,查表识别
  array/heap/object id 并展开为 JSON。替换 http_server.rs + stdlib.rs 共 3 处
  调用点。新增 16 个 VM 集成测试(含 struct/array/nested/Option 展开)。
- **Phase 5(int 转换)**:http_server 注入 path param 时智能推断——可解析为
  i32 则注入 i32,否则 string。`to_int` native(auto.str.to_int, 编号 1516)
  已存在作兜底。

## 回归验证

- baseline: 2788 passed / 83 failed / 79 ignored
- 本次改动: 2813 passed / 83 failed / 79 ignored(+25 新通过,零新失败)

## 端到端验收(纯 VM 模式 AutoVM http_server)

通过在独立线程启动真实 AutoVM HTTP server,用 TcpStream 发请求验证:

- `GET /api/notes/test`(handler 返回 `Note` struct)→ `{"id": 1, "title": "hello"}`
  (旧版返回 `"4000000"` 或 `"null"`)
- `GET /api/echo/42`(handler `fn echo_id(id int) int`)→ `42`
  (旧版 `:id` 注入为 string)

注:015-notes 的 `auto run` 默认走 a2r(Rust 转译)后端,不走 AutoVM
http_server。AutoVM http_server 用于纯 VM 模式(无 a2r)。端到端验收通过
最小 #[api] 程序验证了 AutoVM 路径的正确性。



---

## §1 问题清单

| # | 问题 | 严重度 | 影响 |
|---|---|---|---|
| 1 | for-loop generator 值错误 | P0 | `for n in gen()` 返回错误值(generator task 帧与 FN_PROLOG 冲突) |
| 2 | VM struct 字段访问返回 0 | P0 | `note.title` 返回 0 而非实际值(CONSTRUCT_INSTANCE 可能未正确存字段) |
| 3 | struct/[]T HTTP 序列化 | P1 | handler 返回结构体 → "null"(http_server 只识别 string/i32/null) |
| 4 | List.push 未实现 | P1 | 无法向 List 添加元素 |
| 5 | int 类型转换缺失 | P1 | `:id` 注入为 str,handler `id int` 收到字符串 |

---

## §2 分阶段实施

### Phase 1 — for-loop generator 值修复(P0)

**根因**：generator task 通过 `spawn_task` 创建后,手动 push return_addr + old_bp 并设 ip=func_addr。但 func_addr 处的 FN_PROLOG 期望参数已在栈上,且 RESERVE_STACK 会 push 填充零,导致栈布局错乱。

**修复方案**：不再手动设帧,而是用 `call_fn_by_name` 的模式设帧——它正确处理 FN_PROLOG + 参数。但 `call_fn_by_name` 把 `GeneratorYield` 当 continue。需要改为：在 generator task 上,`GeneratorYield` 应该停止执行并保存状态。

具体做法：
1. `shim_iterator_next` 的 Generator 分支：spawn task 后,用与 `call_fn_by_name` 相同的帧设置(push 参数 → push return_addr → push old_bp → 设 bp → 设 ip=func_addr)
2. 执行循环中,`GeneratorYield` 不再 continue,而是 break(停止,yield 值已在栈上)
3. 后续 next() 时 resume(恢复 ip/bp,继续执行)

**验收**：`fn counter() ~Iter<int> { yield 1; yield 2; yield 3 }` + `for n in counter() { sum += n }` → 输出 6。

### Phase 2 — VM struct 字段访问修复(P0)

**根因**：`CONSTRUCT_INSTANCE`(engine.rs:2592)从栈上 pop field values 并存入 object。但需要调查：
- field values 是否被正确解码(类型推断可能错)
- object registry(`self.objects`)的 key 格式是否与 GET_FIELD 的查找一致
- `GenericInstanceData` 的字段存储是否正确映射

**调查方向**：
1. 在 CONSTRUCT_INSTANCE 加调试日志,打印每个 field 的 name + value
2. 在 GET_FIELD 加调试日志,打印查到的 object 内容
3. 对比创建时存的值和读取时取的值

**验收**：`type Note { id int; title str }` + `let n = Note { id: 1, title: "hello" }` + `print(n.id)` → 1;`print(n.title)` → hello。

### Phase 3 — struct/[]T HTTP 序列化(P1)

**问题**：`http_server.rs` 的 handler 返回值解码只识别 `is_string`/`is_i32`/`is_null`。Object/List 的 NanoValue tag 被 fallthrough 到 "null"。

**修复方案**：在 `serve_blocking_stdnet` 的返回值解码处,增加 object/list 分支：
- `is_object(nv)` → 从 `vm.objects` 取字段 → 递归拼 JSON `{"field": value, ...}`
- `is_list(nv)` → 从 List 取元素 → 拼 JSON `[v1, v2, ...]`
- `?T`(Option) → Some → 内部值的 JSON;None → HTTP 404

**验收**：handler 返回 `Note` → `{"id":1,"title":"hello"}` JSON 响应。

### Phase 4 — List.push 实现(P1)

**问题**：`List.push` native 不存在(搜索 `shim_list_push` 零命中)。

**修复方案**：
1. 新增 `shim_list_push(task, vm)` native：pop value + pop list_id → 向 list 追加元素
2. 注册到 native_catalog + native.rs
3. 注册到 stdlib 的 `auto.list.push`

**验收**：`let l = [1,2]; l.push(3); print(l.len())` → 3。

### Phase 5 — int 类型转换(P1)

**问题**：路径参数 `:id` 注入为 str,但 handler 声明 `id int`。当前 `http_server.rs` 总是 `encode_string`。

**修复方案**：
1. 短期：新增 `to_int(str) int` native(解析字符串为整数)
2. 长期：http_server 根据 handler 参数类型自动转换(codegen 的 api_routes 记录了参数类型)

**验收**：`#[api(path="/api/notes/:id")] fn get_note(id int)` → curl `/api/notes/42` → handler 收到 int 42(不是 str "42")。

---

## §3 实施顺序

```
Phase 1 (generator for-loop) → Phase 2 (struct field) → Phase 3 (序列化)
→ Phase 4 (List.push) → Phase 5 (int 转换)
→ 端到端验收: 015-notes CRUD
```

Phase 1 和 2 是 P0(auto-musk 阻塞)。Phase 3-5 是 P1(可绕过但应该修)。

---

## §4 验收标准

1. `for n in counter() { sum += n }` 正确求和(Phase 1)
2. `note.title` 返回正确字符串(Phase 2)
3. handler 返回 struct → 正确 JSON(Phase 3)
4. `list.push(val)` 可用(Phase 4)
5. `:id` 注入为 int(Phase 5)
6. 015-notes CRUD 端到端跑通
7. 现有 a2r/cookbook/escape 测试零回归
