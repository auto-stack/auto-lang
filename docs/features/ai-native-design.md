# AI-Native Design

Auto is designed from the ground up for AI workloads, with first-class support for node-based dataflow and embedded model inference.

## Node-Based Dataflow

Auto's dataflow system allows you to define computation graphs as code:

```auto
graph image_pipeline {
    input -> preprocess -> model -> postprocess -> output
}
```

Nodes can run on:
- CPU for general computation
- GPU via CUDA/Metal/Vulkan
- NPU for edge AI devices

## Embedded Model Inference

Auto can compile and embed trained models directly into your application:
- ONNX runtime integration
- Custom operators for Auto-specific optimizations
- Quantization support for edge deployment

## Actor-Based Training

Distributed training is naturally expressed using Auto's actor model:
- Each GPU worker is an actor
- Parameter servers communicate via messages
- Fault tolerance through actor supervision trees
