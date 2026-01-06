### 基本类型

- Object类型现在只支持`Vec<Pair>`，需要支持其他表达式，如`if`和`for`
- 临时解决方案：最后一个block必须是object。但是为了通过parser，只能让block支持`,`分隔符。这是不合理的。


### 类型组合

- 现在的组合是直接把部件的`members`和`methods`挂到主类上。可以考虑改为把部件变成主类的一个成员，并添加可以直接调用部件方法的“桥接方法”。或者两种方案都支持（例如引入不同的关键字`has`和`compose`？）

### Spec

用Spec概念把“接口”、“泛型”、“联合类型”、“Concept”等概念融合到一起。

1. 接口形式

```auto
spec Reader {
    fn read(from: Source)
}
```

2. 表达式形式，类似联合类型

```auto
spec Number = int || uint || float || byte
```

3. 判断函数形式

任何参数为`type`类型，且返回类型为`bool`的函数，都可以当作spec来使用。

```auto
spec fn is_ints(T type) bool {
    if T.type.is_array || T.type.is_iter {
        T.type.elem == int
    } else {
        false
    }
}
```

