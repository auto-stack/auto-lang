# Plan: Auto → Godot .tscn 场景生成方案

> **Status**: ✅ Completed(Phase 1-4 全部交付,2026-06)
> **归档**: 本计划已归档至 `docs/plans/archive/`

## Context

a2gd 已能将 Auto 代码转译为 GDScript (.gd)，但一个完整的 Godot 工程还需要 `.tscn` 场景文件来描述节点树结构、属性和信号连接。当前用户需要手写 .tscn 文件（如 `examples/godot/` 中的示例），这阻碍了 Auto→Godot 的自动化流程。

**目标**: 设计一套 Auto 语法，让用户能用 Auto 的节点语法描述 Godot 场景，由编译器自动生成 .tscn 文件。

---

## .tscn 格式分析摘要

一个 Godot 4.x .tscn 文件由以下部分组成：

```
[gd_scene load_steps=N format=3 uid="uid://xxx"]       ← 文件头

[ext_resource type="Script" path="res://foo.gd" id="1"]  ← 外部资源引用
[ext_resource type="Texture2D" path="res://img.png" id="2"]

[sub_resource type="CapsuleShape2D" id="3"]              ← 内嵌子资源
radius = 14.0
height = 30.0

[node name="Player" type="Area2D"]                       ← 根节点
script = ExtResource("1")
z_index = 10

[node name="Sprite" type="AnimatedSprite2D" parent="."]  ← 子节点
sprite_frames = ExtResource("2")

[node name="Collision" type="CollisionShape2D" parent="."]
shape = SubResource("3")

[connection signal="body_entered" from="." to="." method="_on_body_entered"]  ← 信号连接
```

**关键结构**：
| 部分 | 作用 | 生成难度 |
|---|---|---|
| `[gd_scene]` 头 | 元信息 | 简单 |
| `[ext_resource]` | 引用外部文件（脚本、贴图、音频等） | 中等——需自动收集 |
| `[sub_resource]` | 内嵌资源（碰撞形状、材质等） | 中等——需内联定义 |
| `[node]` | 节点树 + 属性 | 核心——映射 Auto 节点语法 |
| `[connection]` | 信号连接 | 简单——声明式 |

---

## 设计方案：`scene` 关键字

### 核心思路

引入新的 `scene` 顶层关键字，使用 Auto 已有的嵌套块语法来描述 Godot 节点树。**场景描述**（→ .tscn）和**游戏逻辑**（→ .gd）可以分离在两个文件中，也可以合并在一个文件中。

### 语法设计

#### 示例 1：最简场景 — Hello World

**Auto 源码** (`hello_scene.at`):
```auto
scene HelloWorld : Control {
    script = "hello.gd"

    node Label "Title" {
        text = "Hello, Godot!"
        position = Vector2(100, 200)
    }
}
```

**生成** `hello_scene.tscn`:
```ini
[gd_scene load_steps=2 format=3 uid="uid://auto_helloworld"]

[ext_resource type="Script" path="res://hello.gd" id="1"]

[node name="HelloWorld" type="Control"]
script = ExtResource("1")

[node name="Title" type="Label" parent="."]
text = "Hello, Godot!"
offset_left = 100.0
offset_top = 200.0
```

#### 示例 2：Dodge the Creeps — Player 场景

**Auto 源码** (`player_scene.at`):
```auto
scene Player : Area2D {
    script = "player.gd"
    z_index = 10

    node AnimatedSprite2D {
        sprite_frames = SpriteFrames {
            animations = [
                {
                    frames: [load("res://art/player_up1.png"), load("res://art/player_up2.png")],
                    loop: true,
                    name: "up",
                    speed: 5.0
                },
                {
                    frames: [load("res://art/player_walk1.png"), load("res://art/player_walk2.png")],
                    loop: true,
                    name: "right",
                    speed: 5.0
                }
            ]
        }
    }

    node CollisionShape2D {
        shape = CapsuleShape2D {
            radius = 5.0
            height = 12.0
        }
    }

    connect body_entered from "." to "." method "_on_body_entered"
}
```

**生成** `player_scene.tscn`:
```ini
[gd_scene load_steps=5 format=3 uid="uid://auto_player"]

[ext_resource type="Script" path="res://player.gd" id="1"]
[ext_resource type="Texture2D" path="res://art/player_up1.png" id="2"]
[ext_resource type="Texture2D" path="res://art/player_up2.png" id="3"]
[ext_resource type="Texture2D" path="res://art/player_walk1.png" id="4"]
[ext_resource type="Texture2D" path="res://art/player_walk2.png" id="5"]

[sub_resource type="SpriteFrames" id="1"]
animations = [ ... ]

[sub_resource type="CapsuleShape2D" id="2"]
radius = 5.0
height = 12.0

[node name="Player" type="Area2D"]
script = ExtResource("1")
z_index = 10

[node name="AnimatedSprite2D" type="AnimatedSprite2D" parent="."]
sprite_frames = SubResource("1")

[node name="CollisionShape2D" type="CollisionShape2D" parent="."]
shape = SubResource("2")

[connection signal="body_entered" from="." to="." method="_on_body_entered"]
```

#### 示例 3：Main 场景 — 含 Timer 和场景实例

**Auto 源码** (`main_scene.at`):
```auto
scene Main : Node {
    script = "main.gd"

    // 场景实例（引用其他 .tscn）
    instance Player "res://player_scene.tscn"

    node Timer "MobTimer" {
        wait_time = 0.5
    }

    node Timer "ScoreTimer" {
        wait_time = 1.0
    }

    node Timer "StartTimer" {
        wait_time = 2.0
        one_shot = true
    }

    instance HUD "res://hud_scene.tscn"

    // 信号连接
    connect timeout from "MobTimer" to "." method "_on_MobTimer_timeout"
    connect timeout from "ScoreTimer" to "." method "_on_ScoreTimer_timeout"
    connect timeout from "StartTimer" to "." method "_on_StartTimer_timeout"
    connect hit from "Player" to "." method "game_over"
    connect start_game from "HUD" to "." method "new_game"
}
```

---

### 语法规则

```
scene <Name> : <GodotType> {
    // 根节点属性
    <prop_name> = <value>

    // 附加脚本
    script = "<path>.gd"

    // 子节点声明
    node <GodotType> ["<InstanceName>"] {
        <prop> = <value>
        ...
    }

    // 嵌套子节点
    node <GodotType> "Parent" {
        node <GodotType> "Child" {
            ...
        }
    }

    // 场景实例化
    instance <Name> "<path>.tscn"

    // 内联子资源
    // (在属性值中使用 TypeName { props } 语法自动生成)

    // 信号连接
    connect <signal> from "<NodePath>" to "<NodePath>" method "<method_name>"
}
```

**属性值类型映射**：
| Auto 值 | .tscn 值 | 示例 |
|---|---|---|
| `123` | `123` | 整数 |
| `3.14` | `3.14` | 浮点 |
| `true` / `false` | `true` / `false` | 布尔 |
| `"text"` | `"text"` | 字符串 |
| `Vector2(100, 200)` | `Vector2(100, 200)` | 向量 |
| `Color(1, 0, 0, 1)` | `Color(1, 0, 0, 1)` | 颜色 |
| `load("res://x.png")` | `ExtResource("N")` | 外部资源 |
| `TypeName { props }` | `SubResource("N")` | 内联子资源 |

### 自动生成规则

1. **ext_resource 自动收集**: 扫描所有 `load("res://...")` 和 `script = "..."` 引用，自动生成 `[ext_resource]` 段
2. **sub_resource 自动分配**: 属性值中使用 `TypeName { props }` 语法的，自动创建 `[sub_resource]` 段并替换引用
3. **uid 生成**: 基于场景名生成确定性 uid（`uid://auto_{name}`）
4. **load_steps 计算**: `1 + ext_resource 数量 + sub_resource 数量`
5. **parent 路径计算**: 嵌套 `node` 的 parent 路径从树结构自动推导

---

## 实现路径

### Phase 1：基础场景生成（✅ 已完成）

**新增/修改的文件**：
- `crates/auto-lang/src/ast/ui.rs` — 添加 `SceneDecl`, `SceneProp`, `SceneNode`, `SceneConnection` AST 类型
- `crates/auto-lang/src/ast.rs` — 添加 `Stmt::SceneDecl` 变体 + Display / to_node / to_atom 实现
- `crates/auto-lang/src/dep.rs`, `infer/stmt.rs`, `indexer.rs` — 为新 Stmt 变体补充 match 分支
- `crates/auto-lang/src/parser.rs` — 添加 `parse_scene_decl()` + `parse_scene_node()` + `parse_scene_instance()` + `parse_scene_connection()`，并用 `looks_like_scene_decl()` 两 token 前瞻识别 `scene` 关键字（不破坏 `scene.foo()` 等表达式）
- `crates/auto-lang/src/trans/tscn.rs` — **新文件**，`TscnGenerator` 生成器（两阶段：先收集 ext_resource，再展平节点树并渲染）
- `crates/auto-lang/src/trans.rs` — 注册 `pub mod tscn`
- `crates/auto-lang/src/lib.rs` — 添加 `trans_tscn(path)` 入口
- `crates/auto/src/main.rs` — 添加 `TransTarget::Tscn`，支持 `auto trans --path scene.at tscn`
- `crates/auto-lang/test/a2gd/tscn/` — 4 个 cookbook 测试（hello / player / timers / nested）

**已实现范围**（超出原 Phase 1 计划）：
- ✅ node 声明 + 基础属性（int/float/bool/string）
- ✅ 构造器值（`Vector2`, `Color`, `Rect2` 等原样输出）
- ✅ `script` 引用 → 自动生成 `ext_resource type="Script"`
- ✅ `instance` 场景实例化 → 自动生成 `ext_resource type="PackedScene"` + `instance=ExtResource(...)`
- ✅ `load("res://...")` 自动资源收集（按扩展名推断类型）
- ✅ `connect` 信号连接
- ✅ 嵌套节点 parent 路径自动推导（`.` → `Level` → `Level/Spawns`）
- ✅ 确定性 uid 生成（FNV-1a → base32，Godot 兼容）
- ✅ `auto_steps` 自动计算（1 + ext_resource 数）

### Phase 2：完整特性

#### Phase 2a：sub_resource 内联定义（✅ 已完成）

属性值中使用 `TypeName { props }` 语法的，自动生成独立的 `[sub_resource]` 段并替换为 `SubResource("N")` 引用。

- `ast/ui.rs`：新增 `SceneValue` 枚举（`Expr` | `SubResource`）与 `SceneSubResource` 结构；`SceneProp.value` 类型由 `Expr` 改为 `SceneValue`
- `parser.rs`：新增 `looks_like_subresource()`（两 token 前瞻识别 `Ident {`）、`parse_scene_value()`、`parse_scene_subresource()`；根节点/子节点/sub_resource 共用 `parse_scene_prop()`
- `trans/tscn.rs`：生成器三阶段化 —— 收集 ext_resource → 分配 sub_resource id（按 AST 地址去重，1 起独立编号）→ 渲染；`load_steps = 1 + ext + sub`；ext/sub id 空间独立（符合 Godot 4）
- 测试：`test/a2gd/tscn/005_subresource/`（CapsuleShape2D + Color 构造器值）

**已知限制**：sub_resource 属性值若是数组/对象字面量（如 SpriteFrames 的 `animations`），目前按 `Debug` 兜底渲染，非 Godot 原生序列化格式 —— 待后续按 Godot 数组/字典格式补全。

#### Phase 2b：a2gd 深度集成 —— 一文件双产物（✅ 已完成）

一个 `.at` 文件可同时包含 `scene` 声明（→ .tscn）与函数/逻辑（→ .gd），由 `auto trans --path X.at godot` 一次产出两个文件，二者通过 `script = "X.gd"` 自动绑定。

- `trans/gdscript.rs`：`Stmt::SceneDecl` 由「报错」改为「跳过」（`Ok(false)`），使混合文件可正常转译为 .gd；并扫描 `SceneDecl` 取其根节点类型作为 `.gd` 的 `extends <Type>`（默认 `Node`），让脚本与场景根类型一致
- `lib.rs`：新增 `trans_godot(path)` —— 先借 `&ast` 生成 .tscn，再 move `ast` 生成 .gd，返回两行产物消息
- `auto/main.rs`：新增 `TransTarget::Godot`，调用 `trans_godot`
- 测试：`test_tscn_006_combined_with_gd` —— 校验 .tscn 含根节点 + 脚本引用、.gd 保留函数且不泄漏 scene、`extends Control`

#### Phase 2c：GDScript 端类型配合（✅ 已完成）

**已完成 —— Godot 内建类型标注（Vector2 等）**

此前 `is_generic_param` 的启发式「空 members/methods 的 `Type::User` 即泛型参数」过于激进，把 Godot 内建类型（Vector2/Color/Node…，其 TypeDecl 同样为空）误判为泛型参数，导致**函数参数与返回值的类型标注被丢弃**（局部变量标注未受影响）。修复：

- `trans/gdscript.rs`：新增 `is_godot_builtin_type(name)` 内建类型白名单（数学类型 / 节点 / 资源）；`is_generic_param` 与 `is_type_decl_generic_param` 的启发式增加该白名单豁免 —— 真正的泛型参数 `T` 仍被丢弃（GDScript 无泛型），Godot 类型标注得以保留
- 测试：`test/a2gd/17_godot_types/001_vector2`（Vector2/Color 在签名中保留标注，与 `08_generics` 中 `T` 被丢弃形成对照）

**`$node`（✅ 已验证 — 无需改动）**

GDScript 的 `$Sprite` 等价于 `get_node("Sprite")`。实测 `get_node("Sprite")`、嵌套路径 `get_node("UI/Label")`、节点字段赋值 `sprite.position = Vector2(...)` 均能原样透传为合法 GDScript。`get_node` 即惯用写法，足以覆盖需求；`$` 简写（需新增 `$` 一元运算符）留作可选增强，非必需。

**`@export`（✅ 已完成）**

`#[export] var speed float = 300.0` → `@export var speed: float = 300.0`。采用 `#[export]` 属性语法（与现有 `#[vm]`/`#[c]`/`#[rs]` 注解体系一致）。改动：

- `ast/store.rs`：`Store` 新增 `attrs: Vec<Name>` 字段（转译器元数据，不参与 `to_atom`/`to_node` 序列化 —— 每次从源码重建，对 VM/IR 无意义）
- 全仓 **39 处** `Store { ... }` 构造点补 `attrs: vec![]`（`parser.rs` 19 + `infer/stmt.rs` 15 + `infer/context.rs`/`scope.rs`/`trans/c.rs`/`vm/codegen.rs` 各 1~2，多为内部 `Meta::Store` 元数据绑定）；通过 `kind: StoreKind::<Variant>,` 这一 Store 专属锚点批量插入
- `parser.rs`：`FnAnnotations` 加 `has_export`；`parse_fn_annotations` 识别 `export`；注解后的 `var`/`let`/`mut`/`const`/`shared` 分支把 `attrs` 透传给 `parse_store_stmt(attrs)`
- `trans/gdscript.rs`：`store()` 检测 `attrs` 含 `export` 时在声明前输出 `@export ` 前缀
- 测试：`test/a2gd/17_godot_types/002_export`（导出变量带 `@export`，非导出变量无前缀）

**Phase 2 全部完成。**

---

### Phase 3：扩展 GDScript 注解（✅ 已完成）

在 `@export` 基础上，支持 Godot 常用变量注解（含带参数的形式）：

```
#[export_range(0, 100, 1)] var hp int = 50       → @export_range(0, 100, 1) var hp: int = 50
#[onready] var sprite = get_node("Sprite")       → @onready var sprite = get_node("Sprite")
#[export_group("Combat")] var damage int = 10    → @export_group("Combat") var damage: int = 10
#[export_enum("A,B,C")] var mode int = 0         → @export_enum("A,B,C") var mode: int = 0
```

设计：复用 Phase 2c 加的 `Store.attrs: Vec<Name>`，**不改类型**——每项存完整注解文本（`"export"` / `"onready"` / `"export_range(0, 100, 1)"`），gdscript 直接逐项输出 `@{text} ` 前缀。

改动：
- `parser.rs`：`FnAnnotations.has_export`（bool）升级为 `store_attrs: Vec<AutoStr>`（完整文本）；新增 `collect_annotation_args()` 辅助函数收集括号内参数文本（含字符串字面量、嵌套括号），同时重构 `derive`/`serde` 旁路改用该辅助函数（消除 ~25 行重复）；`parse_fn_annotations` 对 `export`/`onready`/`export_range`/`export_enum`/`export_group`/`export_subgroup`/`export_flags`/`export_node_path`/`export_file`/`export_dir`/`export_multiline`/`export_color_no_alpha` 收集名+参数
- `trans/gdscript.rs`：`store()` 由「仅 export」改为遍历 `attrs` 逐项输出 `@`-前缀
- 测试：`test/a2gd/17_godot_types/003_annot`（参数化 `export_range`、无参 `onready`、字符串参数 `export_group`、`export` 并存）

验证：trans 304 / gdscript 54 / parser 144 全通过，0 回归（`derive` 重构经 a2r 测试确认无影响）。

#### Phase 3b：脚本级注解 `@tool` / `@icon`（✅ 已完成）

**设计决策**：Phase 3 的注解都是**变量级**（附加在某个 `var` 上，走 `Store.attrs`）。Godot 还有**脚本级**注解——作用于整个脚本、必须出现在 `extends` **之前**。三种可选机制：

- **A（采用）**：`#[tool]`/`#[icon(...)]` 注解，收集到新增的 `Code.file_attrs`，在文件头 `extends` 之前输出
- B：新增 `tool`/`icon` 顶层关键字 —— 引入新关键字成本高，拒绝
- C：复用 `Store.attrs` —— 无目标变量，语义不符，拒绝

关键约束：Godot 要求 `@tool`/`@icon` 在 `extends` 之前，否则编译报错——这是「输出位置必须在 extends 之前」的硬性要求，决定了收集目标必须是文件级（`Code`）而非语句级。

**语法**：
```
#[tool]
#[icon("res://icon.png")]
fn _ready() { ... }
```
→
```gdscript
# Auto-generated from tool.at — do not edit

@tool
@icon("res://icon.png")

extends Node

func _ready():
	...
```

`#[tool]` 后紧跟的声明照常解析（注解不附加到它）——因为 `parse_fn_annotations` 把 tool/icon 推入 parser 级 `self.file_attrs` 而非 `FnAnnotations`，返回后 dispatcher 照常分发后续 `fn`/`var`。

**改动**：
- `ast.rs`：`Code` 新增 `file_attrs: Vec<Name>` 字段（`new()`/`Default` 同步；`Display`/`ToNode`/`AtomWriter`/`ToAtom` 四个 trait 故意不动 —— 与 `Store.attrs` 同理，转译器元数据不序列化）
- `parser.rs`：`Parser` 新增 `file_attrs: Vec<AutoStr>` 字段（3 处构造函数初始化）；`parse_fn_annotations` 新增 `"tool" | "icon"` 分支（复用 Phase 3 的 `collect_annotation_args` 收集参数，推入 `self.file_attrs`）；`parse()` 末尾 `Code { ..., file_attrs: std::mem::take(&mut self.file_attrs) }`
- `trans/gdscript.rs`：`trans()` 头部组装在 `# Auto-generated` 注释之后、`extends` 之前逐行输出 `@{attr}`
- 测试：`test/a2gd/17_godot_types/004_tool`（`#[tool]` + `#[icon("res://icon.png")]` 两注解并排，校验置于 `extends Node` 之前）

**执行步骤（实际已按此完成）**：写夹具 → 运行测试生成 `.wrong.gd` → 核对后改名 `.expected.gd` → 改 parser/gdscript → `cargo build -p auto` → 测试通过 → 回归 trans/parser → 提交。

验证：trans 305 / gdscript 55 / parser 144 全通过，0 回归。

---

### Phase 4：GDScript `signal` 声明（✅ 已完成）

**设计决策**：Godot 的 `signal` 是脚本级声明（出现在 .gd 的 `extends` 之后、成员变量/函数之前）。声明语法的三种可选方案（经与用户确认）：

- A：新增 `signal` 顶层关键字 —— 引入新关键字，拒绝
- B：`#[signal]` 注解挂在无体 fn 上 —— 把信号当函数，语义不符，拒绝
- **C（采用，用户指定）**：`signal` 作为 **scene 体内的特殊节点**，与 `node`/`instance`/`connect` 同为 scene body 元素

关键定位：信号是**脚本数据**（→ .gd），不是**场景数据**（→ .tscn）——所以 `parse_scene_decl` 收集进 `SceneDecl.signals`，tscn 生成器忽略它，gdscript 生成器在 `extends` 之后输出。信号发射 `signal.emit(args)` 与连接 `connect` 均原样透传，无需处理。

**语法**：
```auto
scene Player : Area2D {
    script = "player.gd"

    signal health_changed(new_health int)
    signal game_over

    node CollisionShape2D { ... }
    connect body_entered from "." to "." method "_on_body_entered"
}

fn take_damage(n int) {
    health_changed.emit(n)        // 发射信号，原样透传
}
```
→
```gdscript
extends Area2D

signal health_changed(new_health: int)
signal game_over

func take_damage(n: int):
	health_changed.emit(n)
```

**改动**：
- `ast/ui.rs`：新增 `SceneSignal { name, params: Vec<SceneSignalParam> }` 与 `SceneSignalParam { name, ty }`；`SceneDecl` 新增 `signals: Vec<SceneSignal>` 字段（`pub use ui::*` 自动再导出）
- `parser.rs`：`parse_scene_decl` body 分发新增 `"signal" =>` 分支；新增 `parse_scene_signal()`（解析 `signal name` 或 `signal name(p T, p2 T2)`，参数用 Auto 空格语法，类型用 `parse_type()`）
- `trans/gdscript.rs`：扫描 `SceneDecl` 时同时收集 `signals`（`.cloned()` 以释放 `ast.stmts` 借用），在 `extends` 之后逐行输出 `signal name` / `signal name(p: T, ...)`（类型经 `gdscript_type_name` 映射）
- **顺带修复（发现的 bug）**：`trans()` 的语句分流循环跳过 `Stmt::SceneDecl`（纯元数据）和 `Stmt::EmptyLine`（空行）——二者原本会落入 `main_stmts`，在没有 `fn main()` 时触发多余的空 `func _ready():` 桩。此修复对任意含空行的 .at 文件均生效，非场景专属
- 测试：`test/a2gd/17_godot_types/005_signal`（带参信号 + 无参信号 + 嵌套 node/connect + 顶层 fn 发射信号，校验 `extends Area2D` 后正确输出信号、无空 `_ready` 桩）

**执行步骤（实际已按此完成）**：写夹具 → CLI 试跑发现空 `_ready` 桩 → 临时 debug 打印定位到 `Stmt::EmptyLine` 落入 main_stmts → 加 SceneDecl/EmptyLine 跳过 → 核对输出 → 改名 `.expected.gd` → 回归 → 提交。

验证：trans 306 / gdscript 56 / parser 144（含 tscn 006）全通过，0 回归。

**注**：后续如需支持 Godot 4 的 `await signal`、自定义信号的 `connect`/`disconnect` 调用，可原样透传。

---

### Phase 5：类型化集合与枚举值（✅ 已完成）

GDScript 4 支持类型化集合 `Array[int]` / `Dictionary[String, int]`，以及带显式值的枚举 `enum State { IDLE = 0, RUN = 1 }`。当前 a2gd 在这两处都**丢失了信息**。本阶段分两个独立子任务，均**纯转译层改动，无 AST/parser 变更**，低风险。

#### 调研结论（实施前已核实）

- `trans/gdscript.rs::gdscript_type_name`（≈1427 行）当前：
  - `Type::List(_)` → `"Array"`（**丢弃元素类型**）
  - `Type::Map(_, _)` → `"Dictionary"`（**丢弃 K/V 类型**）
  - `Type::Array(ArrayType)` / `Type::RuntimeArray` / `Type::Slice` → 落入 `_ => "Variant"`（**完全丢失**）
  - `Type::GenericInstance(Future<T>)` 已递归取 inner —— 是现成的递归范例
- `trans/gdscript.rs::enum_decl`（≈853 行）当前：
  - 对每个 `item.name` 强制 `.to_uppercase()`（GDScript 惯例，但与 Auto 源码里的变体名大小写可能不一致）
  - **丢弃** `EnumItem.scalar_value`（`enum State { IDLE = 0 }` → `enum State { IDLE }`）
- AST 已具备全部所需信息：`EnumItem.scalar_value: Option<i32>`；`Type::List(Box<Type>)`、`Type::Map(Box<Type>, Box<Type>)`、`ArrayType`/`SliceType` 含元素类型

---

#### Phase 5a：类型化集合 `Array[T]` / `Dictionary[K, V]`

**设计**：让 `gdscript_type_name` 递归渲染元素类型，与现有 `Future<T>` 递归一致。

| Auto 类型 | 当前输出 | 目标输出 |
|---|---|---|
| `List<int>` | `Array` | `Array[int]` |
| `List<Vector2>` | `Array` | `Array[Vector2]` |
| `Map<str, int>` | `Dictionary` | `Dictionary[String, int]` |
| `[N]int` / `[]int` | `Variant` | `Array[int]` |
| `List<List<int>>` | `Array` | `Array[Array[int]]`（递归） |

GDScript 无定长数组概念，`[N]T` / `[]T` / `[expr]T` 一律映射为 `Array[T]`（元素类型取自 `ArrayType.elem` / `SliceType`，丢失 size 是可接受的，Godot 无对应概念）。

**边界**：元素类型若递归到 `Option`/`Result` → `Variant`，输出 `Array[Variant]`，合法。`Future<T>` 已有递归分支，保持不变。

**改动范围**：仅 `gdscript.rs::gdscript_type_name` 的 3 个 match 分支（`List`/`Map` 改为递归，新增 `Array`/`RuntimeArray`/`Slice` 分支）。无 AST/parser 变更。

**执行步骤（TDD）**：
1. 写测试夹具 `test/a2gd/17_godot_types/006_typed_collections/typed.at`：
   ```auto
   var scores List<int> = [1, 2, 3]
   var lookup Map<str, int> = Map.new()
   fn sum(xs List<int>) int { 0 }
   fn keys(m Map<str, int>) List<str> { [] }
   fn main() { print(scores) }
   ```
2. 运行 `cargo test -p auto-lang --lib test_godot_typed_collections`（先失败：输出 `Array`/`Dictionary`）
3. 改 `gdscript_type_name`：`Type::List(e) => format!("Array[{}]", self.gdscript_type_name(e))`，`Type::Map(k,v) => format!("Dictionary[{}, {}]", …)`，新增 `Array`/`RuntimeArray`/`Slice` 三分支取 elem 递归
4. `cargo build -p auto` → 重跑测试 → 通过；核对 `.expected.gd` 含 `Array[int]` / `Dictionary[String, int]`
5. 回归：`cargo test -p auto-lang --lib -- trans parser`
6. 提交：`feat(a2gd): emit typed Array[T]/Dictionary[K,V] instead of untyped`

**风险**：极低。`gdscript_type_name` 的所有调用点（参数/返回值/变量标注/信号参数）自动受益，无需逐点改。

---

#### Phase 5b：枚举显式值

**设计**：`enum_decl` 渲染时，若 `item.scalar_value` 为 `Some(v)` 则输出 `NAME = v`，否则保持 `NAME`。

| Auto 源码 | 当前输出 | 目标输出 |
|---|---|---|
| `enum State { IDLE = 0, RUN = 1 }` | `enum State { IDLE, RUN }` | `enum State { IDLE = 0, RUN = 1 }` |
| `enum Color { Red, Green }` | `enum Color { RED, GREEN }` | （见下方决策）|

**大小写决策（需用户确认）**：当前强制 `.to_uppercase()` 是 GDScript 惯例，但与 Auto 源码变体名（如 `Color.Red`）不一致——GDScript 端引用会变成 `Color.RED`。两个选项：
- **A（推荐）**：保留源码大小写，`Color.Red` → `Color.Red`，与 Auto 的点访问一致
- **B**：维持现状大写，仅补显式值

选项 A 会改变现有枚举测试的期望输出（若有）。**实施前需 grep 现有 enum 测试期望；若存在且受影响，需用户授权修改期望文件**（遵守 CLAUDE.md 测试期望规则）。

**改动范围**：仅 `gdscript.rs::enum_decl`（≈853 行）。

**执行步骤（TDD）**：
1. grep 现有枚举 a2gd 测试：`find crates/auto-lang/test/a2gd -name '*.expected.gd' | xargs grep -l 'enum '` —— 确认是否有受影响期望
2. 写测试夹具 `test/a2gd/17_godot_types/007_enum_values/enum.at`：
   ```auto
   enum HttpStatus {
       OK = 200
       NotFound = 404
       ServerError = 500
   }
   fn main() { print(HttpStatus.OK) }
   ```
3. 运行测试（先失败：值被丢弃）
4. 改 `enum_decl`：`if let Some(v) = item.scalar_value { write NAME = v } else { NAME }`
5. 按大小写决策（A/B）调整 `item.name` 渲染；若选 A 且有现存期望受影响，向用户申请授权
6. `cargo build -p auto` → 测试通过 → 回归
7. 提交：`feat(a2gd): preserve enum explicit values`

**风险**：低。大小写变更若波及现存期望需授权；纯值保留为增量改动。

---

#### Phase 5 验收标准

- `Array[T]` / `Dictionary[K, V]` 在变量/参数/返回值/信号参数位置均正确输出
- 枚举显式值保留；大小写按用户选定方案一致
- trans / gdscript / parser 全套测试 0 回归
- 两个子任务互相独立，可分别提交

#### Phase 5 实施记录（已完成）

**Phase 5a（类型化集合）**：
- 改动点：`gdscript.rs::gdscript_type_name` 新增递归分支
  - `Type::List(e)` → `Array[{递归 e}]`
  - `Type::Map(k, v)` → `Dictionary[{递归 k}, {递归 v}]`
  - 新增 `Type::Array` / `Type::RuntimeArray` / `Type::Slice` 三分支，取 `.elem` 递归（丢失 size，GDScript 无对应概念）
- 测试夹具：`test/a2gd/17_godot_types/006_typed/typed.at` + `.expected.gd`（测试函数 `test_godot_typed_collections`）
- **连带影响（已处理）**：6 个 cookbook 测试因本改进输出从 `Variant` 升级为 `Array[int]`/`Array[String]`，属正确性提升。经用户授权，已更新对应 `.expected.gd`：`10_collections/001_array_methods`、`cookbook/04_loops/003_for_each`、`cookbook/05_arrays/001_create`、`002_append_pop`、`003_index_access`、`cookbook/06_strings/002_string_array`。

**Phase 5b（枚举显式值 + 大小写）**：
- 用户选定**大小写方案 A**：保留源码大小写，`Color.Red` → GDScript `Color.Red`。已更新 `014_enum/enum.expected.gd`（`RED, GREEN, BLUE` → `Red, Green, Blue`）。
- **发现并解决的问题**：原设计"靠 `scalar_value` 判断是否输出 `= N`"不可行——parser 对无显式值的枚举项会**自动 gap-fill**（`parser.rs` 约 4314-4316 行：仅当 `last_val != 0` 才填补），导致 `enum Direction { Up, Down, Left, Right }` 会被错误渲染为 `Up, Down = 1, Left = 2, Right = 3`。
- **解决方案**：为 `EnumItem` 新增 `value_explicit: bool` 字段（`ast/enums.rs`），**仅当源码写了 `= N` 时为 true**。更新 parser 中全部 4 处 `EnumItem` 构造点（4224/4267/4304/4348 行）传入该字段；gdscript `enum_decl` 仅在 `value_explicit` 时输出 `= N`。
- 测试夹具：`test/a2gd/17_godot_types/007_enum/enum.at` + `.expected.gd`（`HttpStatus` 带显式值 200/404/500；`Direction` 无值枚举保持干净）。
- 改动范围：`ast/enums.rs`（EnumItem 加字段）、`parser.rs`（4 处构造点 + `parse_enum_items`/`parse_scalar_enum_items` 设置 `value_explicit`）、`gdscript.rs::enum_decl`（去掉 `.to_uppercase()`、按 `value_explicit` 门控）。

**回归**：`cargo test -p auto-lang --lib -- trans` → 308 passed / 0 failed。`cargo build -p auto` 通过。



---

## 验证方式

1. ✅ 创建 `test/a2gd/tscn/` 测试目录（5 个用例：hello / player / timers / nested / subresource）
2. ✅ 每个 .at 场景文件 → 生成 .tscn → 与 .expected.tscn 比对（6 个测试全通过，含 uid 确定性）
3. ✅ `auto trans --path scene.at tscn` CLI 端到端验证通过
4. ⬜ 生成的 .tscn 文件在 Godot 4.x 编辑器中打开验证（需人工）
5. ⬜ examples/godot/ 中的手工 .tscn 可替换为自动生成

**测试命令**：
```bash
cargo test -p auto-lang test_tscn        # 5 个 tscn 测试
cargo test -p auto-lang -- trans         # 全部 transpiler 测试（含 tscn + a2gd 等）
```

---

## 待讨论的开放问题

1. **scene 和 fn 能否在同一文件？** 建议先分离：`xxx_scene.at` → .tscn，`xxx.at` → .gd
2. **是否复用 AURA widget 语法？** AURA 的 `col/row/button` 是 UI 抽象，Godot 需要 `Node2D/Area2D/Timer` 等游戏节点。建议用独立的 `scene` 语法，后续再考虑 widget→Godot 桥接
3. **Phase 1 先不支持 sub_resource 内联？** 可以先用手写的 ext_resource 引用，Phase 2 再加内联
