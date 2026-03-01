# 前后端通讯架构设计

## 概述

AutoLang 支持多种 UI 编译路线，不同路线的前后端通讯机制不同。本文档定义统一的 API 设计策略，使开发者只需编写一套代码，编译器自动生成适配各平台的通讯层。

---

## 核心原则

```
┌─────────────────────────────────────────────────────────────────┐
│                    是否需要 API 抽象层？                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  需要 ──→ 前端在浏览器中运行 ──→ 必须通过 HTTP 通讯            │
│  │                                                              │
│  └── 只有 a2vue                                                │
│                                                                 │
│  不需要 ──→ 前后端同一进程 ──→ 直接函数调用                     │
│  │                                                              │
│  └── a2rust, a2jet, a2lvgl                                     │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 四条路线的通讯架构

### 1. a2vue（Web 技术）

**唯一需要 API 抽象层的路线**，因为需要支持两种部署模式。

```
┌─────────────────────────────────────────────────────────────────┐
│                        a2vue 架构                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Auto API 定义（只写一套）                                      │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  #[api]                                                  │   │
│  │  fn get_user(id int) User { ... }                       │   │
│  │                                                         │   │
│  │  #[api]                                                  │   │
│  │  fn save_file(path str, content str) void { ... }       │   │
│  └─────────────────────────────────────────────────────────┘   │
│                           │                                     │
│                           ▼                                     │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              编译器自动生成三套代码                       │   │
│  │                                                         │   │
│  │  api-interface.ts  → TypeScript 接口定义                 │   │
│  │  api-tauri.ts      → Tauri IPC 实现（单机模式）          │   │
│  │  api-http.ts       → HTTP API 实现（Web 模式）           │   │
│  │  api.ts            → 环境检测 + 自动选择                 │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  后端（Auto → Rust）                                            │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              编译器自动生成双模式代码                     │   │
│  │                                                         │   │
│  │  单机模式:  #[tauri::command] fn get_user(...)         │   │
│  │  Web 模式:   async fn get_user(...) + axum 路由        │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  部署方式：                                                     │
│  ├── 单机：Tauri App（WebView + IPC，无网络）                  │
│  └── Web：Browser + HTTP Server（需要网络）                    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

#### 单机模式 vs Web 模式

| 特性 | 单机模式 (Tauri) | Web 模式 |
|-----|-----------------|---------|
| **通讯方式** | IPC（进程内） | HTTP/WebSocket |
| **延迟** | 极低（微秒级） | 有网络延迟（毫秒级） |
| **认证** | 不需要 | 需要 (JWT/Session) |
| **CORS** | 无限制 | 需要配置 |
| **文件系统** | 直接访问 | 通过 API |
| **部署** | 安装包 | 服务器 |

---

### 2. a2rust（原生 Desktop）

**不需要 API 抽象层**，前后端直接调用。

```
┌─────────────────────────────────────────────────────────────────┐
│                        a2rust 架构                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Auto 代码（前后端融合）                                        │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  // 后端逻辑                                              │   │
│  │  fn get_user(id int) User {                              │   │
│  │      User { id: id, name: "Alice" }                      │   │
│  │  }                                                       │   │
│  │                                                         │   │
│  │  // 前端 UI                                              │   │
│  │  fn UserCard(user_id int) Widget {                      │   │
│  │      let user = get_user(user_id)  // 直接调用           │   │
│  │      Card::new().child(Text::new(user.name))            │   │
│  │  }                                                       │   │
│  └─────────────────────────────────────────────────────────┘   │
│                           │                                     │
│                           ▼                                     │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  编译为 Rust 代码（同一编译单元）                         │   │
│  │                                                         │   │
│  │  pub fn get_user(id: i32) -> User { ... }              │   │
│  │                                                         │   │
│  │  fn user_card(user_id: i32) -> Widget {                │   │
│  │      let user = get_user(user_id);  // 直接函数调用     │   │
│  │      ...                                                 │   │
│  │  }                                                       │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  部署方式：                                                     │
│  └── 只有单机模式（原生应用）                                   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

### 3. a2jet（Android）

**通常不需要 API 抽象层**，但联网应用可能需要 HTTP。

```
┌─────────────────────────────────────────────────────────────────┐
│                        a2jet 架构                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Auto 代码 → Jetpack Compose (Kotlin)                          │
│                                                                 │
│  后端选择：                                                     │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                                                         │   │
│  │  方案 A：纯本地 App（离线）                              │   │
│  │  ├── 后端: Auto → Kotlin 代码                          │   │
│  │  ├── 通讯: 直接函数调用                                │   │
│  │  └── 场景: 计算器、笔记、本地工具                       │   │
│  │                                                         │   │
│  │  方案 B：联网 App（在线）                               │   │
│  │  ├── 后端: 远程服务器 (Auto → Rust HTTP Server)        │   │
│  │  ├── 通讯: HTTP API                                    │   │
│  │  └── 场景: 社交、电商、云存储                           │   │
│  │                                                         │   │
│  │  方案 C：混合 App                                       │   │
│  │  ├── 本地后端: 部分功能离线可用                         │   │
│  │  ├── 远程后端: 部分功能需要联网                         │   │
│  │  └── 场景: 音乐播放器（本地播放 + 云端同步）            │   │
│  │                                                         │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  部署方式：                                                     │
│  ├── 本地：APK 内直接调用                                       │
│  └── 联网：HTTP API 调用                                        │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

### 4. a2lvgl（嵌入式）

**不需要 API 抽象层**，前后端完全融合。

```
┌─────────────────────────────────────────────────────────────────┐
│                        a2lvgl 架构                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Auto 代码 → C 代码（单一固件）                                 │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  // 后端逻辑                                              │   │
│  │  fn read_sensor(pin int) int {                          │   │
│  │      gpio_read(pin)                                      │   │
│  │  }                                                       │   │
│  │                                                         │   │
│  │  // 前端 UI                                              │   │
│  │  fn SensorDisplay(pin int) Widget {                     │   │
│  │      let value = read_sensor(pin)  // 直接调用           │   │
│  │      Text::new(f"Sensor: {value}")                      │   │
│  │  }                                                       │   │
│  └─────────────────────────────────────────────────────────┘   │
│                           │                                     │
│                           ▼                                     │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  编译为 C 代码（同一固件）                                │   │
│  │                                                         │   │
│  │  int read_sensor(int pin) { ... }                       │   │
│  │                                                         │   │
│  │  lv_obj_t* sensor_display(int pin) {                    │   │
│  │      int value = read_sensor(pin);  // 直接调用         │   │
│  │      ...                                                 │   │
│  │  }                                                       │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  部署方式：                                                     │
│  └── 只有单机模式（嵌入式固件）                                 │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Auto API 定义语法

### 基本语法

```auto
// api/user.at

/// 用户信息
type User = {
    id: int
    name: str
    email: str
}

/// 获取用户信息
#[api]
fn get_user(id int) User {
    db.find_user(id)
}

/// 保存文件
#[api]
fn save_file(path str, content str) void {
    fs.write(path, content)
}

/// 获取文件列表
#[api(name = "listFiles")
fn list_files(dir str) List<str> {
    fs.list_dir(dir)
}
```

### API 注解

```auto
// 自定义 API 名称（用于生成 TypeScript 函数名）
#[api(name = "getUserById")]
fn get_user(id int) User { ... }

// REST 路径映射（Web 模式专用）
#[api(method = "GET", path = "/users/:id")]
fn get_user(id int) User { ... }

#[api(method = "POST", path = "/files")]
fn save_file(path str, content str) void { ... }

// 缓存控制
#[api(cache = 60)]
fn get_config() Config { ... }

// 权限控制
#[api(auth = true)]
fn delete_user(id int) void { ... }
```

---

## 编译器生成规则

### a2vue 前端生成

#### TypeScript 接口定义

```typescript
// api-interface.ts (自动生成)

export interface User {
    id: number
    name: string
    email: string
}

export interface IApi {
    getUser(id: number): Promise<User>
    saveFile(path: string, content: string): Promise<void>
    listFiles(dir: string): Promise<string[]>
}
```

#### Tauri IPC 实现

```typescript
// api-tauri.ts (自动生成)

import { invoke } from '@tauri-apps/api/tauri'
import type { IApi, User } from './api-interface'

export const tauriApi: IApi = {
    getUser: (id) => invoke<User>('get_user', { id }),
    saveFile: (path, content) => invoke<void>('save_file', { path, content }),
    listFiles: (dir) => invoke<string[]>('list_files', { dir }),
}
```

#### HTTP API 实现

```typescript
// api-http.ts (自动生成)

import axios from 'axios'
import type { IApi, User } from './api-interface'

const BASE_URL = '/api'

export const httpApi: IApi = {
    getUser: async (id) => {
        const res = await axios.get<User>(`${BASE_URL}/users/${id}`)
        return res.data
    },
    saveFile: async (path, content) => {
        await axios.post(`${BASE_URL}/files`, { path, content })
    },
    listFiles: async (dir) => {
        const res = await axios.get<string[]>(`${BASE_URL}/files`, { params: { dir } })
        return res.data
    },
}
```

#### 环境检测

```typescript
// api.ts (自动生成)

import { tauriApi } from './api-tauri'
import { httpApi } from './api-http'
import type { IApi } from './api-interface'

const isTauri = typeof window !== 'undefined' && '__TAURI__' in window

export const api: IApi = isTauri ? tauriApi : httpApi
```

### a2vue 后端生成

#### Tauri 命令模式

```rust
// 单机模式后端 (自动生成)

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

#[tauri::command]
pub fn get_user(id: i32) -> User {
    db::find_user(id)
}

#[tauri::command]
pub fn save_file(path: String, content: String) {
    fs::write(path, content).unwrap()
}

#[tauri::command]
pub fn list_files(dir: String) -> Vec<String> {
    fs::read_dir(dir).unwrap()
        .map(|e| e.unwrap().path().to_str().unwrap().to_string())
        .collect()
}

// 注册命令
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_user,
            save_file,
            list_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

#### HTTP Server 模式

```rust
// Web 模式后端 (自动生成)

use axum::{
    routing::{get, post},
    Json, Router, extract::Path, extract::Query,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

// API 处理函数
async fn get_user(Path(id): Path<i32>) -> Json<User> {
    Json(db::find_user(id))
}

async fn save_file(Json(payload): Json<SaveFileRequest>) -> &'static str {
    fs::write(&payload.path, &payload.content).unwrap();
    "ok"
}

async fn list_files(Query(params): Query<ListFilesQuery>) -> Json<Vec<String>> {
    let files = fs::read_dir(&params.dir).unwrap()
        .map(|e| e.unwrap().path().to_str().unwrap().to_string())
        .collect();
    Json(files)
}

// 路由定义
pub fn api_routes() -> Router {
    Router::new()
        .route("/users/:id", get(get_user))
        .route("/files", post(save_file))
        .route("/files", get(list_files))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .nest("/api", api_routes())
        .layer(tower_http::cors::CorsLayer::permissive());

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

### a2rust 生成

```rust
// a2rust 模式 (自动生成)

// 后端函数
pub fn get_user(id: i32) -> User {
    db::find_user(id)
}

pub fn save_file(path: &str, content: &str) {
    fs::write(path, content).unwrap()
}

// 前端组件直接调用
fn user_card(user_id: i32) -> Widget {
    let user = get_user(user_id);  // 直接函数调用，无需 IPC/HTTP
    Card::new()
        .child(Text::new(&user.name))
}
```

### a2jet 生成

```kotlin
// a2jet 模式 (自动生成)

// 后端函数
suspend fun getUser(id: Int): User {
    return db.findUser(id)
}

suspend fun saveFile(path: String, content: String) {
    fs.write(path, content)
}

// 前端组件直接调用
@Composable
fun UserCard(userId: Int) {
    val user = remember { mutableStateOf<User?>(null) }

    LaunchedEffect(userId) {
        user.value = getUser(userId)  // 直接调用
    }

    user.value?.let {
        Card {
            Text(text = it.name)
        }
    }
}
```

### a2lvgl 生成

```c
// a2lvgl 模式 (自动生成)

// 后端函数
User get_user(int id) {
    return db_find_user(id);
}

void save_file(const char* path, const char* content) {
    fs_write(path, content);
}

// 前端组件直接调用
lv_obj_t* user_card_create(lv_obj_t* parent, int user_id) {
    User user = get_user(user_id);  // 直接调用

    lv_obj_t* card = lv_obj_create(parent);
    lv_obj_t* label = lv_label_create(card);
    lv_label_set_text(label, user.name);
    return card;
}
```

---

## 对比总结

| 路线 | 前端技术 | 后端技术 | 通讯方式 | API 抽象层 | Web 模式 |
|-----|---------|---------|---------|-----------|---------|
| **a2vue** | Vue/TypeScript | Rust | IPC / HTTP | ✅ 需要 | ✅ 支持 |
| **a2rust** | AutoUI (Rust) | Rust | 直接调用 | ❌ 不需要 | ❌ 不支持 |
| **a2jet** | Jetpack Compose | Kotlin | 直接调用 / HTTP | ⚠️ 可选 | ❌ 不支持* |
| **a2lvgl** | LVGL (C) | C | 直接调用 | ❌ 不需要 | ❌ 不支持 |

*a2jet 可能有联网场景，但不是 "Web 模式"（不是浏览器部署）

---

## 模块结构

```
crates/auto-lang/src/api/
├── mod.rs              # API 编译器入口
├── parser.rs           # 解析 #[api] 注解
├── types.rs            # API 类型定义
└── targets/
    ├── mod.rs          # Target trait
    ├── vue.rs          # a2vue: interface + tauri + http
    ├── rust.rs         # a2rust: 直接 Rust 函数
    ├── kotlin.rs       # a2jet: Kotlin 函数
    └── c.rs            # a2lvgl: C 函数
```

---

## 前置条件

本设计依赖以下未实现的功能：

1. **Web Server 后端** - Auto → Rust HTTP Server (axum/actix)
2. **API 注解解析** - 解析 `#[api]` 及其参数
3. **TypeScript 类型生成** - Auto 类型 → TypeScript 类型
4. **Kotlin 后端支持** - Auto → Kotlin (可选)

---

## 相关文档

- [Design Token 系统](./design-token-system.md)
- [a2c + LVGL 架构分析](./a2c-lvgl-analysis.md)
- [Plan 101: DevTools 与热重载](../plans/101-devtools-hotreload.md)
