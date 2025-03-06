## Atom

Atom是Auto语言的一个子集，用来传递数据。

如果把Auto语言看做是“动态的数据”，那么Atom就相当于是Auto语言“凝聚”之后的产物。

Atom相当于JSON和XML的结合体。

## 名称的由来

Atom的意思是“AuTo Object Markup”，即用来表示Auto语言数据对象的标记语言。

本来我打算仿照JSON，取名为ASON（Auto Script Object Notation），但发现已经有同名且功能类似的项目了，
因此改为了Aton（AuTo Object Notation），又发现这个词容易被误读为Atom，因此干脆改为Atom。

## Atom的结构

Atom是树状结构，且是JSON的超集。也就是说所有的JSON数据都是合法的Atom数据。

例如：

```json
// 数组
[1, 2, 3]

// 对象
{
    "name": "Puming",
    "age": 41
}

// 数组和对象的组合
[
    {
        "name": "Puming",
        "age": 41,
        "skills": ["Rust", "C", "Python", "JavaScript"]
    }
]
```

Atom的基本数据类型也和JSON类似：

```json
// 字符串
"Hello, World!"

// 数字
123

// 布尔值
true
false

// 空值
null
```

相比于JSON，Atom扩展了如下特点：

1. 区分整数和小数

```json
{
    "a": 1, // 整数
    "b": 2.2,  // 小数
    "c": 3.0f, // 浮点数
    "d": 4.5e-10 // 科学计数法
}
```

2. 支持注释，这在作为配置文件时非常有用

```json
// 单行注释
{
    "a": 1, // 整数
    "b": 2.2,  // 小数
    "c": 3.0f // 浮点数
}

/* 多行注释
    多行注释
    多行注释
*/
```

3. 对象的健可以不用双引号，这一点和Javascript的对象类似：

```js
{
    a: "1",
    b: "2",
}
```

4. 换行可以替代逗号：

```js
[
    1 // 注意，可以不用逗号
    2
    3
]

{
    a: "1"
    b: "2"
    c: "3"
}
```

5. 根节点可以是数组，也可以是对象，也可以是数组和对象的组合：

```js
[
    1,
    2,
    3
]

{
    "a": 1,
    "b": 2,
    "c": 3
}
```

6. 对象中的键值对（pair）可以独立存在：

```
name: "Puming"
age: 41

{
    other: "other"
    props: ["props"]
}
```

## 节点

如果只是增加了上面的特性，那么Atom和JSON的区别也不大，最多相当于JOSN5。

Atom真正有特色的地方，是在类JSON的语法结构中，增加了类似XML的“节点”概念。

下面是XML的节点：

```xml
<root id="123">
    <name>Puming</name>
    <age>41</age>
</root>
```

上面的`<root>`、`<name>`、`<age>`都是节点。

如果用JSON来表示的话，由于JSON没法直接“插入子节点”，只能用对象和数组的组合来表示：

```json
{
    "node": "root",
    "id": "123",
    "children": [
        {
            "node": "name",
            "value": "Puming"
        },
        {
            "node": "age",
            "value": 41
        }
    ]
}
```

Atom的节点语法如下：

```js
root(args) {
    subnode() {...}
    ...
}
```

这个语法和函数调用有点像，但是增加了`{}`，表示其子节点。

上面的例子用Atom的节点语法表示如下：

```js
root(id:"123") {
    name("Puming")
    age(41)
}
```

注意，子节点中空的`{}`在Atom中是可以省略的，但是由于Auto语言中有函数调用的语法，容易混淆，
所以在Auto语言中节点的`{}`是不能省略的。

如果需要更深层次的子节点，继续加`{}`即可：

```js
root(id:"123") {
    name("Puming") {
        surname("Zhao") {
            ...
        }
    }
    age(41)
}
```

相比于JSON和XML，Atom的节点语法信息表达能力是相同的，但是更为简洁，语法风格也更为统一。

最重要的是，Atom的语法结构是Auto语言的子集，因此和Auto语言的结合最为自然。

## Auto与Atom的转换

Auto语言在好几个场景中都是当做“动态配置语言”来使用的，比如：

- 作为Automan编译工具的配置语言
- 作为AutoGen模版工具的模版语言
- 作为AutoUI界面库的UI描述语言

这些语言中都有很多“动态”的内容，如变量、函数、循环、导入等，
但是经过编译器的处理，最终所有的动态数据都会被解析出来，成为静态的数据。

这个静态的数据，表示成文本的格式，就是Atom。

例如，如下的AutoGen模版：

```js

// 读取学生的成绩
var students = read_scores(...)

// 生成学生的成绩报告
for s in students {
    student(name:s.name, age:s.age) {
        for c in s.courses {
            course(name:c.name, score:c.score)
        }
    }
}
```


经过编译器处理后，会生成如下的Atom：

```js
student(name: "Zhang San", age: 10) {
    course(name: "Math", score: 95)
    course(name: "English", score: 85)
    course(name: "Chinese", score: 90)
}

student(name: "Li Si", age: 11) {
    course(name: "Math", score: 90)
    course(name: "English", score: 80)
    course(name: "Chinese", score: 95)
}

// ...
```

这样的Atom数据，就可以很容易地转换为JSON/XML/YAML等格式，
或者传递给其他工具处理。

Auto语言自带的Atom解析库，可以把Atom数据解析为两种不同的形式：

1. 带类型的对象。这需要有Atom对应的Schema数据。
2. 动态的对象。和JSON转换为JS的Object类似。

前者可以当做结构体来直接访问，例如上面的Atom就会生成`Student`数组，并且每个`Student`结构体都有一个成员是`Course`的数组。

```auto
type Student {
    name str
    age int
    courses []Course
}

type Course {
    name str
    score int
}
```

后者则可以利用动态的访问语法来取值：

```js
var students = read_scores_from_atom(...)

students.len // 可以获取数量
students[5].courses[3].score // 可以获取第5个学生的第1门课程的名称

students[1000].courses[5000].score // 这里数组越界了，会提前直接返回null
```

Auto语言的Atom解析库，类似于XML的XPath，可以有多种解析方式。

