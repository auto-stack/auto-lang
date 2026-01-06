## OS

Auto语言践行LaOS（Languange as OS）理念，因此在语言层面支持一套“虚拟的”操作系统。

为此，Auto语言提供如下OS概念：

1. Os.Process：于普通操作系统中的“进程”。
2. Os.Thread: 操作系统中的“线程”。
3. Task: 类比与操作系统中的“协程”或“纤程”。

我们在平时编程时，只需要与`Task`打交道即可，因此在Auto语言里，提供了专门的`task`关键字。

`task`的签名和`type`很像，区别在于`task`内定义的元数默认的寿元是`@Task`级别。

```auto
task blink {
  mut color = Red

  on 10ms {
    // toggle color
    if color == Red {
      pins.led.red = true
      pins.led.blue = false
      color = Blue
    } else {
      pins.led.red = true
      pins.led.blue = false
      color = Red
    }
  }
}
```

这个例子中，`blink`任务会持续进行，每`10ms`切换一次颜色。
这里的`color`变量，其寿元为`@Task`级别，即任务结束时，才会被销毁。

在使用中，我们通过`task.start`方法来启动一个任务，通过`task.end`方法来结束一个任务。

```auto
fn main {
  // 启动任务
  let t = blink.start()
  // ...
  中途可以获取任务的状态
  print(t.color)
  // 结束任务
  t.end()
}
```
