# HTTP Server 标准库设计

## 概述

本文档分析在 AutoLang 中实现 HTTP Server 所需的标准库模块和 API。Auto 有两种执行机制：

1. **AutoVM** - 动态解释器，通过 FFI 调用 Rust
2. **a2r** - 转译器，直接编译为 Rust

标准库通过 `.at` + `.vm.at`（VM 模式）或 `.at` + `.rs.at`（转译模式）实现。

---

## 模块层次架构

```
┌─────────────────────────────────────────────────────────────────┐
│                    HTTP Server 模块层次                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  应用层                                                         │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  http.Server                                             │   │
│  │  ├── route() 路由注册                                    │   │
│  │  ├── middleware() 中间件                                 │   │
│  │  └── listen() 启动监听                                   │   │
│  └─────────────────────────────────────────────────────────┘   │
│                           │                                     │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  http.Request / http.Response                           │   │
│  │  ├── Header 操作                                         │   │
│  │  ├── Body 读写                                           │   │
│  │  ├── Cookie 操作                                         │   │
│  │  └── Query/Path 参数                                     │   │
│  └─────────────────────────────────────────────────────────┘   │
│                           │                                     │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  json / form / url                                       │   │
│  │  ├── encode() / decode()                                 │   │
│  │  └── 类型转换                                            │   │
│  └─────────────────────────────────────────────────────────┘   │
│                           │                                     │
│  基础设施层                                                     │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  net.TcpListener / net.TcpStream                        │   │
│  │  ├── bind() 绑定地址                                     │   │
│  │  ├── accept() 接受连接                                   │   │
│  │  └── read/write 数据传输                                 │   │
│  └─────────────────────────────────────────────────────────┘   │
│                           │                                     │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  async runtime (tokio)                                   │   │
│  │  ├── spawn() 任务创建                                    │   │
│  │  ├── async/await 语法                                    │   │
│  │  └── Channel 通讯                                        │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 模块依赖关系

```
                    ┌──────────┐
                    │   http   │
                    └────┬─────┘
                         │
         ┌───────────────┼───────────────┐
         │               │               │
         ▼               ▼               ▼
    ┌─────────┐    ┌─────────┐    ┌─────────┐
    │   net   │    │  json   │    │   url   │
    └────┬────┘    └─────────┘    └─────────┘
         │
         ▼
    ┌─────────┐
    │  async  │  ← 最底层，所有 I/O 依赖
    └────┬────┘
         │
         ├───────────────┐
         │               │
         ▼               ▼
    ┌─────────┐    ┌─────────┐
    │   log   │    │   env   │
    └─────────┘    └─────────┘
```

---

## 模块详细设计

### 1. 异步运行时模块 (`async`)

**最底层的模块**，HTTP Server 必须支持异步 I/O。

#### 文件结构

```
stdlib/auto/
├── async.at            # Auto API 定义
├── async.vm.at         # VM 实现绑定
└── async.rs.at         # a2r 转译绑定
```

#### API 定义

```auto
// async.at

// ═══════════════════════════════════════════════════════════
// 异步任务
// ═══════════════════════════════════════════════════════════

/// 创建异步任务
fn async.spawn(f fn()) void

/// 等待所有任务完成
fn async.wait_all() void

/// 获取当前任务数量
fn async.task_count() int

// ═══════════════════════════════════════════════════════════
// Future 类型
// ═══════════════════════════════════════════════════════════

type Future<T>

/// 创建已完成的 Future
fn async.ready<T>(value T) Future<T>

/// 等待 Future 完成
fn Future<T>.await() T

/// 映射 Future 结果
fn Future<T>.map<U>(f fn(T) U) Future<U>

/// 组合多个 Future
fn async.all<T>(futures List<Future<T>>) Future<List<T>>

// ═══════════════════════════════════════════════════════════
// Channel（任务间通讯）
// ═══════════════════════════════════════════════════════════

type Sender<T>
type Receiver<T>

/// 创建 Channel
fn async.channel<T>() (Sender<T>, Receiver<T>)

/// 发送数据
fn Sender<T>.send(value T) void

/// 接收数据（阻塞）
fn Receiver<T>.recv() T

/// 尝试接收（非阻塞）
fn Receiver<T>.try_recv() T?

/// 获取发送端克隆
fn Sender<T>.clone() Sender<T>

/// 关闭发送端
fn Sender<T>.close() void

/// 检查是否已关闭
fn Receiver<T>.is_closed() bool

// ═══════════════════════════════════════════════════════════
// 同步原语
// ═══════════════════════════════════════════════════════════

type Mutex<T>

/// 创建互斥锁
fn async.mutex<T>(value T) Mutex<T>

/// 锁定并访问
fn Mutex<T>.lock<U>(f fn(&T) U) U

type RwLock<T>

/// 创建读写锁
fn async.rwlock<T>(value T) RwLock<T>

/// 读锁定
fn RwLock<T>.read<U>(f fn(&T) U) U

/// 写锁定
fn RwLock<T>.write<U>(f fn(&mut T) U) U

type Condvar

/// 创建条件变量
fn async.condvar() Condvar

/// 等待通知
fn Condvar.wait(mutex Mutex) void

/// 通知一个等待者
fn Condvar.notify_one() void

/// 通知所有等待者
fn Condvar.notify_all() void
```

#### Rust 后端绑定示例

```rust
// async.vm.at (VM 绑定)

#[vm]
fn async_spawn(f: VmFunc) {
    tokio::spawn(async move {
        f.call();
    });
}

#[vm]
fn async_channel<T>() -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = tokio::sync::mpsc::channel::<T>(16);
    (Sender(tx), Receiver(rx))
}

#[vm]
fn sender_send<T>(sender: &Sender<T>, value: T) {
    sender.0.blocking_send(value).unwrap();
}

#[vm]
fn receiver_recv<T>(receiver: &mut Receiver<T>) -> T {
    receiver.0.blocking_recv().unwrap()
}
```

---

### 2. 网络模块 (`net`)

TCP 网络通信的基础模块。

#### 文件结构

```
stdlib/auto/
├── net.at              # Auto API 定义
├── net.vm.at           # VM 实现绑定
└── net.rs.at           # a2r 转译绑定
```

#### API 定义

```auto
// net.at

// ═══════════════════════════════════════════════════════════
// TCP 监听器
// ═══════════════════════════════════════════════════════════

type TcpListener

/// 绑定地址并创建监听器
fn net.tcp_bind(addr str) TcpListener?

/// 接受新连接（阻塞）
fn TcpListener.accept() TcpStream?

/// 接受新连接（非阻塞，返回 Future）
fn TcpListener.accept_async() Future<TcpStream?>

/// 获取本地地址
fn TcpListener.local_addr() str

/// 设置非阻塞模式
fn TcpListener.set_nonblocking(nonblock bool) void

// ═══════════════════════════════════════════════════════════
// TCP 流
// ═══════════════════════════════════════════════════════════

type TcpStream

/// 读取数据到 buffer，返回实际读取字节数
fn TcpStream.read(buf []byte) int

/// 写入数据，返回实际写入字节数
fn TcpStream.write(data []byte) int

/// 读取所有数据直到 EOF
fn TcpStream.read_all() []byte

/// 写入所有数据
fn TcpStream.write_all(data []byte) void

/// 关闭连接
fn TcpStream.close() void

/// 关闭写入端
fn TcpStream.shutdown_write() void

/// 设置读取超时（毫秒）
fn TcpStream.set_read_timeout(ms int) void

/// 设置写入超时（毫秒）
fn TcpStream.set_write_timeout(ms int) void

/// 设置非阻塞模式
fn TcpStream.set_nonblocking(nonblock bool) void

/// 获取对端地址
fn TcpStream.peer_addr() str

/// 获取本地地址
fn TcpStream.local_addr() str

// ═══════════════════════════════════════════════════════════
// 异步读写
// ═══════════════════════════════════════════════════════════

/// 异步读取
fn TcpStream.read_async(buf []byte) Future<int>

/// 异步写入
fn TcpStream.write_async(data []byte) Future<int>

/// 异步读取所有数据
fn TcpStream.read_all_async() Future<[]byte>

// ═══════════════════════════════════════════════════════════
// 地址解析
// ═══════════════════════════════════════════════════════════

type SocketAddr

/// 解析地址 (如 "127.0.0.1:8080")
fn net.parse_addr(addr str) SocketAddr?

/// 从主机名和端口创建地址
fn net.socket_addr(host str, port int) SocketAddr?

/// 获取 IP
fn SocketAddr.ip() str

/// 获取端口
fn SocketAddr.port() int

/// 转换为字符串
fn SocketAddr.to_string() str

// ═══════════════════════════════════════════════════════════
// DNS 解析
// ═══════════════════════════════════════════════════════════

/// 解析主机名获取 IP 地址列表
fn net.resolve(host str) List<str>

/// 解析主机名并连接
fn net.tcp_connect(host str, port int) TcpStream?

/// 异步连接
fn net.tcp_connect_async(host str, port int) Future<TcpStream?>
```

---

### 3. HTTP 模块 (`http`)

HTTP 协议处理，Server 和 Client 支持。

#### 文件结构

```
stdlib/auto/
├── http.at             # Auto API 定义
├── http.vm.at          # VM 实现绑定
└── http.rs.at          # a2r 转译绑定
```

#### API 定义

```auto
// http.at

// ═══════════════════════════════════════════════════════════
// HTTP Server
// ═══════════════════════════════════════════════════════════

type Server

/// 创建 HTTP Server
fn http.server() Server

/// 添加路由 GET
fn Server.get(path str, handler fn(Request) Response) Server

/// 添加路由 POST
fn Server.post(path str, handler fn(Request) Response) Server

/// 添加路由 PUT
fn Server.put(path str, handler fn(Request) Response) Server

/// 添加路由 DELETE
fn Server.delete(path str, handler fn(Request) Response) Server

/// 添加路由 PATCH
fn Server.patch(path str, handler fn(Request) Response) Server

/// 添加路由 HEAD
fn Server.head(path str, handler fn(Request) Response) Server

/// 添加路由 OPTIONS
fn Server.options(path str, handler fn(Request) Response) Server

/// 添加通用路由
fn Server.route(method str, path str, handler fn(Request) Response) Server

/// 添加静态文件路由
fn Server.static(prefix str, dir str) Server

/// 添加中间件
fn Server.use(middleware fn(Request, fn() Response) Response) Server

/// 设置 404 处理器
fn Server.not_found(handler fn(Request) Response) Server

/// 设置错误处理器
fn Server.on_error(handler fn(Error) Response) Server

/// 启动监听（阻塞）
fn Server.listen(addr str) void

/// 启动监听（带回调）
fn Server.listen_with(addr str, on_start fn()) void

/// 优雅关闭
fn Server.shutdown() void

/// 获取服务器地址
fn Server.addr() str

// ═══════════════════════════════════════════════════════════
// HTTP Request
// ═══════════════════════════════════════════════════════════

type Request

/// 获取请求方法 (GET/POST/PUT/DELETE...)
fn Request.method() str

/// 获取请求路径
fn Request.path() str

/// 获取完整 URL
fn Request.url() str

/// 获取查询参数
fn Request.query(key str) str

/// 获取查询参数（带默认值）
fn Request.query_or(key str, default str) str

/// 获取所有查询参数
fn Request.queries() Map<str, str>

/// 获取路径参数 (如 /users/:id)
fn Request.param(key str) str

/// 获取所有路径参数
fn Request.params() Map<str, str>

/// 获取 Header
fn Request.header(key str) str

/// 获取所有 Headers
fn Request.headers() Map<str, str>

/// 获取 Body (原始字节)
fn Request.body() []byte

/// 获取 Body (字符串)
fn Request.text() str

/// 获取 Body (JSON 解析)
fn Request.json<T>() T

/// 获取 Body (表单解析)
fn Request.form() Map<str, str>

/// 获取 Content-Type
fn Request.content_type() str

/// 获取 Content-Length
fn Request.content_length() int

/// 获取 Cookie
fn Request.cookie(name str) str

/// 获取所有 Cookies
fn Request.cookies() Map<str, str>

/// 获取客户端 IP
fn Request.remote_addr() str

/// 检查是否为 HTTPS
fn Request.is_secure() bool

/// 获取 Host
fn Request.host() str

// ═══════════════════════════════════════════════════════════
// HTTP Response
// ═══════════════════════════════════════════════════════════

type Response

/// 创建响应
fn http.response() Response

/// 设置状态码
fn Response.status(code int) Response

/// 获取状态码
fn Response.status_code() int

/// 设置 Header
fn Response.header(key str, value str) Response

/// 设置多个 Headers
fn Response.headers(headers Map<str, str>) Response

/// 设置 Content-Type
fn Response.content_type(ct str) Response

/// 设置 Body (文本)
fn Response.text(body str) Response

/// 设置 Body (JSON)
fn Response.json(data T) Response

/// 设置 Body (HTML)
fn Response.html(body str) Response

/// 设置 Body (原始字节)
fn Response.bytes(data []byte) Response

/// 设置 Body (文件)
fn Response.file(path str) Response

/// 设置 Body (流)
fn Response.stream(reader Reader) Response

/// 设置 Cookie
fn Response.cookie(name str, value str) Response

/// 设置 Cookie (带选项)
fn Response.cookie_with(name str, value str, opts CookieOptions) Response

/// 删除 Cookie
fn Response.clear_cookie(name str) Response

/// 重定向 (302)
fn Response.redirect(url str) Response

/// 重定向 (指定状态码)
fn Response.redirect_with(url str, code int) Response

/// 设置响应大小（用于 Content-Length）
fn Response.size(bytes int) Response

// ═══════════════════════════════════════════════════════════
// 预定义响应
// ═══════════════════════════════════════════════════════════

/// 200 OK
fn http.ok(body str) Response

/// 200 OK (JSON)
fn http.ok_json(data T) Response

/// 201 Created
fn http.created(data T) Response

/// 204 No Content
fn http.no_content() Response

/// 400 Bad Request
fn http.bad_request(msg str) Response

/// 401 Unauthorized
fn http.unauthorized(msg str) Response

/// 403 Forbidden
fn http.forbidden(msg str) Response

/// 404 Not Found
fn http.not_found(msg str) Response

/// 500 Internal Server Error
fn http.internal_error(msg str) Response

/// 503 Service Unavailable
fn http.service_unavailable(msg str) Response

// ═══════════════════════════════════════════════════════════
// Cookie 选项
// ═══════════════════════════════════════════════════════════

type CookieOptions = {
    max_age: int       // 秒数，-1 表示 Session Cookie
    expires: str       // 过期时间 (HTTP 日期格式)
    path: str          // 默认 "/"
    domain: str        // 默认空
    secure: bool       // 仅 HTTPS
    http_only: bool    // 禁止 JS 访问
    same_site: str     // "Strict" | "Lax" | "None"
}

// ═══════════════════════════════════════════════════════════
// HTTP Client
// ═══════════════════════════════════════════════════════════

type HttpClient

/// 创建 HTTP Client
fn http.client() HttpClient

/// 设置超时（毫秒）
fn HttpClient.timeout(ms int) HttpClient

/// 设置 Base URL
fn HttpClient.base_url(url str) HttpClient

/// 设置默认 Header
fn HttpClient.header(key str, value str) HttpClient

/// GET 请求
fn HttpClient.get(url str) HttpResponse

/// POST 请求
fn HttpClient.post(url str, body str) HttpResponse

/// POST JSON
fn HttpClient.post_json(url str, data T) HttpResponse

/// 异步 GET
fn HttpClient.get_async(url str) Future<HttpResponse>

/// 异步 POST
fn HttpClient.post_async(url str, body str) Future<HttpResponse>

// ═══════════════════════════════════════════════════════════
// HTTP Response (Client)
// ═══════════════════════════════════════════════════════════

type HttpResponse

/// 获取状态码
fn HttpResponse.status() int

/// 判断是否成功 (2xx)
fn HttpResponse.is_success() bool

/// 获取 Header
fn HttpResponse.header(key str) str

/// 获取所有 Headers
fn HttpResponse.headers() Map<str, str>

/// 获取 Body (文本)
fn HttpResponse.text() str

/// 获取 Body (JSON)
fn HttpResponse.json<T>() T

/// 获取 Body (字节)
fn HttpResponse.bytes() []byte
```

#### 使用示例

```auto
// examples/http_server.at

use auto.http
use auto.json

// 用户数据类型
type User = {
    id: int
    name: str
    email: str
}

// 模拟数据库
let users = [
    User { id: 1, name: "Alice", email: "alice@example.com" },
    User { id: 2, name: "Bob", email: "bob@example.com" },
]

fn main() {
    let server = http.server()

    // GET / - 首页
    server.get("/", fn(req Request) Response {
        http.ok("Welcome to Auto HTTP Server!")
    })

    // GET /users - 获取用户列表
    server.get("/users", fn(req Request) Response {
        http.ok_json(users)
    })

    // GET /users/:id - 获取单个用户
    server.get("/users/:id", fn(req Request) Response {
        let id = req.param("id").to_int()
        for user in users {
            if user.id == id {
                return http.ok_json(user)
            }
        }
        http.not_found("User not found")
    })

    // POST /users - 创建用户
    server.post("/users", fn(req Request) Response {
        let user = req.json<User>()
        // 保存用户...
        http.created(user)
    })

    // 静态文件
    server.static("/static", "./public")

    // 启动服务器
    print("Server running on http://127.0.0.1:8080")
    server.listen("127.0.0.1:8080")
}
```

---

### 4. JSON 模块 (`json`)

JSON 编解码支持。

#### 文件结构

```
stdlib/auto/
├── json.at             # Auto API 定义
├── json.vm.at          # VM 实现绑定
└── json.rs.at          # a2r 转译绑定
```

#### API 定义

```auto
// json.at

// ═══════════════════════════════════════════════════════════
// JSON 编解码
// ═══════════════════════════════════════════════════════════

/// 将值编码为 JSON 字符串
fn json.encode(value T) str

/// 将 JSON 字符串解码为值
fn json.decode<T>(s str) T

/// 将值编码为 JSON 字节
fn json.encode_bytes(value T) []byte

/// 将 JSON 字节解码为值
fn json.decode_bytes<T>(data []byte) T

/// 格式化 JSON（美化输出）
fn json.prettify(s str) str

/// 压缩 JSON（移除空白）
fn json.minify(s str) str

/// 检查是否为有效 JSON
fn json.is_valid(s str) bool

// ═══════════════════════════════════════════════════════════
// JSON Value 类型（动态解析）
// ═══════════════════════════════════════════════════════════

type JsonValue

/// 从字符串解析为 JsonValue
fn json.parse(s str) JsonValue?

/// 从 JsonValue 转为字符串
fn JsonValue.to_string() str

/// 获取类型
fn JsonValue.type() str  // "null" | "bool" | "number" | "string" | "array" | "object"

/// 检查类型
fn JsonValue.is_null() bool
fn JsonValue.is_bool() bool
fn JsonValue.is_number() bool
fn JsonValue.is_string() bool
fn JsonValue.is_array() bool
fn JsonValue.is_object() bool

/// 转换为基本类型
fn JsonValue.as_bool() bool
fn JsonValue.as_number() float
fn JsonValue.as_int() int
fn JsonValue.as_string() str

/// 转换为集合类型
fn JsonValue.as_array() List<JsonValue>
fn JsonValue.as_object() Map<str, JsonValue>

/// 获取对象字段
fn JsonValue.get(key str) JsonValue?

/// 设置对象字段
fn JsonValue.set(key str, value JsonValue) void

/// 获取数组元素
fn JsonValue.at(index int) JsonValue?

/// 获取数组/对象长度
fn JsonValue.len() int

/// 遍历数组
fn JsonValue.each(f fn(int, JsonValue)) void

/// 遍历对象
fn JsonValue.each_key(f fn(str, JsonValue)) void

// ═══════════════════════════════════════════════════════════
// JSON 构建器
// ═══════════════════════════════════════════════════════════

type JsonBuilder

/// 创建 JSON 对象构建器
fn json.object() JsonBuilder

/// 创建 JSON 数组构建器
fn json.array() JsonBuilder

/// 添加字段
fn JsonBuilder.field(key str, value T) JsonBuilder

/// 添加元素（数组）
fn JsonBuilder.push(value T) JsonBuilder

/// 构建 JsonValue
fn JsonBuilder.build() JsonValue

/// 构建字符串
fn JsonBuilder.to_string() str
```

---

### 5. URL 编码模块 (`url`)

URL 编解码和解析。

#### 文件结构

```
stdlib/auto/
├── url.at              # Auto API 定义
├── url.vm.at           # VM 实现绑定
└── url.rs.at           # a2r 转译绑定
```

#### API 定义

```auto
// url.at

// ═══════════════════════════════════════════════════════════
// URL 编解码
// ═══════════════════════════════════════════════════════════

/// URL 编码（完整编码）
fn url.encode(s str) str

/// URL 解码
fn url.decode(s str) str

/// 编码查询参数组件（编码空格为 +）
fn url.encode_component(s str) str

/// 解码查询参数组件
fn url.decode_component(s str) str

/// 编码查询参数
fn url.encode_query(params Map<str, str>) str

/// 解码查询参数
fn url.decode_query(query str) Map<str, str>

/// 解码查询参数（支持多值）
fn url.decode_query_multi(query str) Map<str, List<str>>

// ═══════════════════════════════════════════════════════════
// URL 解析
// ═══════════════════════════════════════════════════════════

type Url

/// 解析 URL
fn url.parse(s str) Url?

/// 从组件构建 URL
fn url.build(scheme str, host str, port int, path str, query str, fragment str) Url

/// 获取协议
fn Url.scheme() str

/// 获取用户名
fn Url.username() str

/// 获取密码
fn Url.password() str

/// 获取主机名
fn Url.host() str

/// 获取端口
fn Url.port() int

/// 获取默认端口
fn Url.default_port() int

/// 获取路径
fn Url.path() str

/// 获取路径片段
fn Url.path_segments() List<str>

/// 获取查询字符串
fn Url.query() str

/// 获取解析后的查询参数
fn Url.query_params() Map<str, str>

/// 获取 Fragment
fn Url.fragment() str

/// 检查是否为绝对 URL
fn Url.is_absolute() bool

/// 检查是否为相对 URL
fn Url.is_relative() bool

/// Join 相对路径
fn Url.join(relative str) Url

/// 转换为字符串
fn Url.to_string() str

// ═══════════════════════════════════════════════════════════
// URL Builder
// ═══════════════════════════════════════════════════════════

type UrlBuilder

/// 创建 URL 构建器
fn url.builder() UrlBuilder

/// 设置协议
fn UrlBuilder.scheme(s str) UrlBuilder

/// 设置主机
fn UrlBuilder.host(s str) UrlBuilder

/// 设置端口
fn UrlBuilder.port(n int) UrlBuilder

/// 设置路径
fn UrlBuilder.path(s str) UrlBuilder

/// 添加路径片段
fn UrlBuilder.path_segment(s str) UrlBuilder

/// 设置查询参数
fn UrlBuilder.query(key str, value str) UrlBuilder

/// 设置 Fragment
fn UrlBuilder.fragment(s str) UrlBuilder

/// 构建 URL
fn UrlBuilder.build() Url

/// 构建字符串
fn UrlBuilder.to_string() str
```

---

### 6. 日志模块 (`log`)

结构化日志支持。

#### 文件结构

```
stdlib/auto/
├── log.at              # Auto API 定义
├── log.vm.at           # VM 实现绑定
└── log.rs.at           # a2r 转译绑定
```

#### API 定义

```auto
// log.at

// ═══════════════════════════════════════════════════════════
// 基本日志
// ═══════════════════════════════════════════════════════════

/// 调试日志
fn log.debug(msg str) void

/// 信息日志
fn log.info(msg str) void

/// 警告日志
fn log.warn(msg str) void

/// 错误日志
fn log.error(msg str) void

/// 致命错误（记录后退出）
fn log.fatal(msg str) void

// ═══════════════════════════════════════════════════════════
// 格式化日志
// ═══════════════════════════════════════════════════════════

/// 格式化调试
fn log.debugf(format str, args ...) void

/// 格式化信息
fn log.infof(format str, args ...) void

/// 格式化警告
fn log.warnf(format str, args ...) void

/// 格式化错误
fn log.errorf(format str, args ...) void

// ═══════════════════════════════════════════════════════════
// 配置
// ═══════════════════════════════════════════════════════════

/// 日志级别
type LogLevel = "debug" | "info" | "warn" | "error" | "fatal"

/// 设置日志级别
fn log.set_level(level LogLevel) void

/// 获取当前日志级别
fn log.level() LogLevel

/// 设置日志格式
/// 占位符: {time} {level} {msg} {file} {line} {module}
fn log.set_format(format str) void

/// 设置输出目标
fn log.set_output(target str) void  // "stdout" | "stderr" | 文件路径

/// 启用/禁用颜色
fn log.set_color(enabled bool) void

/// 启用/禁用时间戳
fn log.set_timestamp(enabled bool) void

// ═══════════════════════════════════════════════════════════
// 结构化日志
// ═══════════════════════════════════════════════════════════

type Logger

/// 创建命名 Logger
fn log.logger(name str) Logger

/// 带字段的日志
fn Logger.with_field(key str, value T) Logger

/// 带多个字段的日志
fn Logger.with_fields(fields Map<str, T>) Logger

/// 日志方法
fn Logger.debug(msg str) void
fn Logger.info(msg str) void
fn Logger.warn(msg str) void
fn Logger.error(msg str) void
```

---

### 7. 环境模块 (`env`)

环境变量和进程管理。

#### 文件结构

```
stdlib/auto/
├── env.at              # Auto API 定义
├── env.vm.at           # VM 实现绑定
└── env.rs.at           # a2r 转译绑定
```

#### API 定义

```auto
// env.at

// ═══════════════════════════════════════════════════════════
// 环境变量
// ═══════════════════════════════════════════════════════════

/// 获取环境变量
fn env.get(key str) str?

/// 获取环境变量（带默认值）
fn env.get_or(key str, default str) str

/// 设置环境变量
fn env.set(key str, value str) void

/// 删除环境变量
fn env.remove(key str) void

/// 获取所有环境变量
fn env.all() Map<str, str>

/// 检查环境变量是否存在
fn env.has(key str) bool

// ═══════════════════════════════════════════════════════════
// 工作目录
// ═══════════════════════════════════════════════════════════

/// 获取当前工作目录
fn env.cwd() str

/// 设置当前工作目录
fn env.chdir(path str) void

// ═══════════════════════════════════════════════════════════
// 命令行参数
// ═══════════════════════════════════════════════════════════

/// 获取所有命令行参数
fn env.args() List<str>

/// 获取程序名
fn env.program() str

/// 获取参数（跳过程序名）
fn env.argv() List<str>

// ═══════════════════════════════════════════════════════════
// 进程控制
// ═══════════════════════════════════════════════════════════

/// 退出程序
fn env.exit(code int) void

/// 正常退出 (code=0)
fn env.quit() void

/// 获取进程 ID
fn env.pid() int

/// 获取父进程 ID
fn env.ppid() int

// ═══════════════════════════════════════════════════════════
// 系统信息
// ═══════════════════════════════════════════════════════════

/// 获取主机名
fn env.hostname() str

/// 获取操作系统
fn env.os() str  // "windows" | "linux" | "macos"

/// 获取架构
fn env.arch() str  // "x86_64" | "aarch64"

/// 获取用户名
fn env.username() str

/// 获取家目录
fn env.home_dir() str?

/// 获取临时目录
fn env.temp_dir() str

// ═══════════════════════════════════════════════════════════
// 配置文件
// ═══════════════════════════════════════════════════════════

/// 加载 .env 文件
fn env.load_file(path str) void

/// 加载默认 .env
fn env.load() void
```

---

## 语言特性需求

除了标准库模块，Auto 语言本身还需要以下特性支持：

### 1. 闭包捕获

```auto
// 当前可能不支持
let greeting = "Hello"
server.get("/", fn(req Request) Response {
    http.ok(greeting)  // 捕获外部变量
})
```

### 2. 泛型方法

```auto
// 需要支持泛型方法调用
fn json.decode<T>(s str) T

let user = json.decode<User>(json_str)
let users = req.json<List<User>>()
```

### 3. 异步语法（可选，但推荐）

```auto
// 方案 A：回调风格（当前可行）
server.get("/", fn(req Request) Response {
    http.ok("Hello")
})

// 方案 B：async/await 语法（未来）
server.get("/", async fn(req Request) Response {
    let data = await db.query("SELECT ...")
    http.json(data)
})
```

### 4. 错误处理

```auto
// 需要统一的错误处理机制
fn tcp_bind(addr str) TcpListener? {
    // 返回 Option 类型
}

// 或使用 Result 类型
fn tcp_bind(addr str) Result<TcpListener, Error> {
    // ...
}
```

### 5. 方法链（Builder 模式）

```auto
// 支持链式调用
http.response()
    .status(200)
    .header("Content-Type", "application/json")
    .json(data)
```

---

## 实现优先级

| 优先级 | 模块 | 原因 |
|-------|------|------|
| **P0** | `async` | HTTP Server 的基础，必须先实现 |
| **P0** | `net` | TCP 监听是 Server 的核心 |
| **P1** | `http` | HTTP 协议处理 |
| **P1** | `json` | REST API 必需 |
| **P2** | `url` | Query 参数处理 |
| **P2** | `log` | 调试和运维 |
| **P3** | `env` | 配置管理 |

---

## 文件结构总览

```
stdlib/auto/
├── async.at            # 异步运行时 API
├── async.vm.at         # VM 实现
├── async.rs.at         # a2r 实现
│
├── net.at              # 网络 API
├── net.vm.at
├── net.rs.at
│
├── http.at             # HTTP API
├── http.vm.at
├── http.rs.at
│
├── json.at             # JSON API
├── json.vm.at
├── json.rs.at
│
├── url.at              # URL API
├── url.vm.at
├── url.rs.at
│
├── log.at              # 日志 API
├── log.vm.at
├── log.rs.at
│
├── env.at              # 环境变量 API
├── env.vm.at
└── env.rs.at
```

---

## 相关文档

- [前后端通讯架构设计](./frontend-backend-communication.md)
- [Design Token 系统](./design-token-system.md)
- [Plan 100: a2js → a2ts 移植计划](../plans/100-a2js-to-a2ts.md)
