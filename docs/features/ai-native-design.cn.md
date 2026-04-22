# AI 原生设计

Auto 从底层开始为 AI 工作负载而设计，对基于节点的数据流和嵌入式模型推理提供一流支持。

## 基于节点的数据流

Auto 的数据流系统允许你将计算图定义为代码：

```auto
graph image_pipeline {
    input -> preprocess -> model -> postprocess -> output
}
```

节点可以运行在：
- CPU 上进行通用计算
- GPU 上通过 CUDA/Metal/Vulkan
- NPU 上进行边缘 AI 设备推理

## 嵌入式模型推理

Auto 可以直接将训练好的模型编译并嵌入到你的应用程序中：
- ONNX 运行时集成
- Auto 特定优化的自定义算子
- 边缘部署的量化支持

## 基于 Actor 的训练

分布式训练自然地通过 Auto 的 Actor 模型表达：
- 每个 GPU 工作器是一个 Actor
- 参数服务器通过消息通信
- 通过 Actor 监督树实现容错
