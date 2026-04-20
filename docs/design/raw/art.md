这是一个关于 **Auto Runtime (ART)** 的核心架构设计文档。这份文档将指导后续 `a2c` 转译器后端库的开发，以及 AutoMan 构建逻辑的编写。

---

# Auto Runtime (ART) 跨平台架构设计文档 (v1.0)

**日期**: 2026-02-04
**模块**: Auto Runtime (ART) / Standard Library
**目标**: 实现 Auto 语言在 Desktop (Win/Linux/Mac) 和 Embedded (RTOS/Bare Metal) 上的“真跨平台”。

---

## 1. 架构总览 (Executive Summary)

Auto Runtime (ART) 并非重写标准 C 库 (libc)，而是一个 **“智能垫片层 (Smart Shim Layer)”**。它向下屏蔽 OS 和 libc 的差异，向上为 Auto 语言提供统一的原语。

核心战略为 **“编译期多态 (Compile-Time Polymorphism)”**：

1. **逻辑层**：不依赖系统的纯算法（如字符串、切片管理），由 Auto 实现或生成纯 C 代码。
2. **适配层**：依赖系统的 IO/并发，通过 AutoMan 在编译时注入不同的 C 实现文件。
3. **依赖管理**：通过源码内嵌 (Vendoring) 解决第三方库依赖问题。

---

## 2. 详细分层设计 (Layered Design)

ART 由上至下分为四层。

### Layer 1: Auto Native Core (逻辑核心)

* **定位**: 平台无关的纯逻辑。
* **依赖**: 无 (Freestanding)。
* **内容**:
* `String` / `Slice` / `Vector` 的内存布局与操作。
* `Option` / `Result` 的解包逻辑。
* UTF-8 编解码、数学库 (`libm` wrapper)、哈希算法。


* **实现**: 由 Auto 源码编写，转译为标准 C99 代码。

### Layer 2: Platform Abstraction Layer (PAL / 适配层)

* **定位**: 统一系统调用接口 (System Call Shim)。
* **机制**: 定义一套 `art_` 开头的 C API，不同平台提供不同实现。
* **接口示例**:
```c
// art_pal.h
typedef void* art_file_t;
typedef void* art_thread_t;

// IO
int art_fs_open(const char* path, int mode, art_file_t* out_file);
// Allocator
void* art_alloc(size_t size);

```



### Layer 3: Capability Guard (能力卫士)

* **定位**: 静态裁剪机制。
* **机制**: 配合 AutoMan 的 `[capabilities]` 配置。如果目标平台不支持某项能力（如 `fs`），则对应的 Auto 模块 (`std.fs`) 在编译阶段会被“门控”拦截，防止生成调用了未定义符号的 C 代码。

### Layer 4: Backend Implementation (后端实现)

* **定位**: 真正的脏活累活。
* **分类**:
* **backend_posix**: 映射到 `pthread`, `socket`, `fopen` (Linux/Mac)。
* **backend_win32**: 映射到 `CreateThread`, `Winsock`, `CreateFile` (Windows)。
* **backend_rtos**: 映射到 `FreeRTOS`/`ThreadX` API, `LwIP` (MCU)。
* **backend_bare**: 映射到 `Newlib-stub`, `HAL` (裸机)。



---

## 3. 关键子系统设计

### 3.1 内存管理 (Memory Subsystem)

不强制实现分配器，而是路由分配请求。

* **API**: `art_alloc`, `art_free`, `art_realloc`
* **策略**:
* **Desktop**: 直接 `#define art_alloc malloc` (或链接 mimalloc)。
* **RTOS**: 路由至 `pvPortMalloc`。
* **Bare Metal**: 如果无堆，提供一个基于静态大数组 (`static uint8_t heap[SIZE]`) 的简易 Bump Pointer 分配器。



### 3.2 异步并发模型 (Unified Async Model)

这是最复杂的适配部分。Auto 语言层统一使用 `go func()` (Task) 和 `chan` (Channel)。

**统一 ABI (在 C 中生成):**

```c
// 启动任务
void art_async_spawn(void (*entry)(void*), void* arg, size_t stack_size);
// 通道操作 (阻塞)
int art_async_chan_send(art_chan_t ch, void* msg);
int art_async_chan_recv(art_chan_t ch, void* out_msg);
// 让出 CPU
void art_async_yield();

```

**后端适配策略:**

| 特性 | Desktop (High Performance) | MCU (Real-time / Low Resource) |
| --- | --- | --- |
| **底层实现** | **Libuv (Event Loop) + Libcoro (Stack Switching)** | **Native RTOS Task (1:1 Mapping)** |
| **IO 模型** | 非阻塞 IO + Epoll/IOCP | 阻塞 IO (由 RTOS 调度器挂起) |
| **Task 成本** | 极低 (用户态协程) | 中等 (需要分配独立栈空间) |
| **Channel** | 内存队列 (无锁或互斥锁) | RTOS Queue / Mailbox |
| **Preemption** | 协作式 (Cooperative) | 抢占式 (Preemptive) |

**设计理由**: 在 MCU 上强行跑 Libuv+Coro 开销太大且浪费了 RTOS 现成的调度器。1:1 映射虽然消耗栈内存，但最稳定且调试方便。

### 3.3 第三方库管理 (Vendoring Strategy)

解决 "DLL Hell" 和 "Header missing" 的核心策略。

* **核心库 (System Dependent)**: 如 Socket, Thread。由 ART 包含多套 `.c` 实现。
* **功能库 (System Independent)**: 如 `SDL3`, `cJSON`, `sqlite3`。
* **策略**: **Source Vendoring (源码内嵌)**。
* AutoMan 的仓库中保存这些库的 C 源码（或 git submodule）。
* 编译时，AutoMan 生成 `build.ninja`，将 `vendor/sdl3/src/*.c` 加入编译列表。
* 自动定义必要的宏（如 `SDL_VIDEO_DRIVER_WINDOWS`）。



---

## 4. AutoMan 配合机制 (Build System Integration)

AutoMan 是 ART 的指挥官。它负责读取配置并选择正确的文件。

### 4.1 配置文件 (`App.auto` / `config.auto`)

利用 Auto 语言的 Config 模式描述目标平台能力。

```auto
// target_stm32f4.auto
target {
    arch: "cortex-m4",
    os: "freertos",
    
    // 关键：能力定义 (Features / Capabilities)
    caps: {
        fs: false,           // 禁用文件系统模块
        net: "lwip",         // 网络使用 LwIP 适配
        threading: "rtos",   // 使用 RTOS 原生线程
        float: "hard",       // 硬件浮点
        display: "spi_lcd"   // 显示后端
    }
}

```

### 4.2 编译流水线 (Build Pipeline)

1. **解析配置**: AutoMan 读取 Target Config。
2. **依赖树裁剪**:
* 用户代码 `import std.fs`。
* AutoMan 检查 `caps.fs` 为 `false`。
* **Action**: 抛出编译错误 *"Module 'std.fs' is not available on target 'stm32f4'"*。


3. **源文件注入**:
* `caps.threading == "rtos"` -> 注入 `art/backends/freertos/sched.c`。
* `caps.net == "lwip"` -> 注入 `art/backends/lwip/net.c` 并添加 LwIP include 路径。


4. **宏定义生成**:
* 生成 `art_config.h`，包含 `#define ART_PLATFORM_RTOS 1` 等。


5. **C 编译**: 调用 GCC/Clang 编译所有注入的 `.c` 和生成的 `.c`。

---

## 5. 目录结构规范 (Directory Structure)

```text
/lib/std (Auto 标准库源码)
  /core         (纯 Auto 代码)
  /io           (依赖 PAL 的 Auto 代码)
  /net
  /sync

/runtime (ART C 语言运行时)
  /include
    art_pal.h   (PAL 接口定义)
    art_def.h   (类型定义)

  /common       (通用 C 实现)
    art_string.c
    art_math.c

  /vendor       (第三方库源码镜像)
    /sdl3
    /mbedtls
    /sqlite

  /backends     (PAL 实现 - 由 AutoMan 选择性编译)
    /posix      (Linux/macOS)
      art_io_posix.c
      art_net_bsd.c
      art_sched_uv.c   (+libuv +libcoro)
    
    /win32      (Windows)
      art_io_win32.c
      art_net_winsock.c
      art_sched_uv.c
      
    /freertos   (RTOS Adapter)
      art_io_dummy.c   (或 fatfs 适配)
      art_sched_rtos.c (xTaskCreate 包装)

```

---

## 6. 实施路线图 (Roadmap)

1. **Phase 1 (Skeleton)**:
* 定义 `art_pal.h`。
* 实现 `backend_posix` (最小化 stdio, malloc)。
* 跑通 Hello World。


2. **Phase 2 (Async Desktop)**:
* 集成 `libuv` 和 `libcoro`。
* 实现 `go func()` 在 PC 上的调度。


3. **Phase 3 (Async RTOS)**:
* 实现 `backend_freertos`。
* 验证 `go func()` 映射到 `xTaskCreate` 的稳定性。


4. **Phase 4 (Ecosystem)**:
* 集成 SDL3 和 LwIP。
* 完善 AutoMan 的 Capability 检查逻辑。



---

**设计结论**:
通过将 libc 视为单纯的“宿主环境”而非“依赖项”，并通过 AutoMan 在编译前进行精细的源文件选择和宏控制，Auto 可以避免陷入重写 libc 的泥潭，同时在 PC 上获得高性能异步 IO，在 MCU 上获得原生级别的实时性与低开销。
