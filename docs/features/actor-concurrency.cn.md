# Actor 并发

Auto 基于 Actor 模型构建并发系统，让并发编程天生安全。

## Actor 模型

Actor 是通过消息传递进行通信的独立计算单元。每个 Actor 具有以下特性：
- 外部无法直接访问的私有状态
- 用于接收消息的邮箱
- 创建新 Actor 的能力

## 异步类型

Auto 引入了 `~T` 类型表示异步值。这使得类型系统可以显式地表明某个值可能是并发计算的。

```auto
fn process(actor: ~Actor) -> ~Result {
    actor.send(message)
}
```

## 安全保证

- Actor 之间不存在共享可变状态
- 消息传递是唯一的通信渠道
- 数据竞争在设计上就是不可能的
