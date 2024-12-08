### 基本类型

- Object类型现在只支持`Vec<Pair>`，需要支持其他表达式，如`if`和`for`
- 临时解决方案：最后一个block必须是object。但是为了通过parser，只能让block支持`,`分隔符。这是不合理的。