# AutoVM 解释器

AutoVM 是 Auto 的专用虚拟机和解释器，旨在在所有支持的平台上高效运行 Auto 代码。

## 架构

AutoVM 使用基于寄存器的字节码虚拟机，具有：
- 用于生产构建的提前编译（AOT）
- 用于热路径的即时编译（JIT）
- 用于快速开发迭代的增量编译

## 热重载

在开发过程中，AutoVM 支持代码修改的热重载，无需重新启动应用程序。这适用于：
- 函数体更新
- 类型定义变更
- Actor 消息处理器

## 跨平台

AutoVM 运行于：
- 桌面：Windows、macOS、Linux
- 移动：iOS、Android
- 嵌入式：无操作系统的裸机
- Web：通过 WebAssembly 转译

## 调试

AutoVM 包含内置调试器，具有：
- 断点和单步执行
- 变量检查
- 内存分析
- Actor 消息跟踪
