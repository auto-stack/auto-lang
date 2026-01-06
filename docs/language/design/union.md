## Union Types

### C语言

在C语言中，`union`（联合体）是一种内存复用的方案，即同一片内存可以当作不同的类型来访问。

实际使用中，我们常常不直接用`union`，而是添加一个字段来表示具体取哪一个，即`tagged-union`。

```c
typedef enum {
    KIND_INT,
    KIND_FLOAT,
    KIND_CHAR,
} MyUnionKind;

typedef struct {
    // tag
    MyUnionKind kind;
    // data
    union {
        int i;
        float f;
        char c;
    } as;
} MyUnion;
```

这样在实际使用时，可以通过判断`tag`值来选取具体的数据类型。

```c
if (my_union.kind == KIND_INT) {
    printf("int: %d\n", my_union.as.i);
} else if (my_union.kind == KIND_FLOAT) {
    printf("float: %f\n", my_union.as.f);
} else if (my_union.kind == KIND_CHAR) {
    printf("char: %c\n", my_union.as.c);
}
```

也可以通过`switch`语句来根据`tag`值取用对应的数据：

```C
switch (my_union.kind) {
    case KIND_INT:
        printf("int: %d\n", my_union.as.i);
        break;
    case KIND_FLOAT:
        printf("float: %f\n", my_union.as.f);
        break;
    case KIND_CHAR:
        printf("char: %c\n", my_union.as.c);
        break;
}
```

### Rust语言

在Rust语言中，除了与C差不多的原始`union`类型，还提供了`tagged_union`的直接支持
即`typed enum`：

```rust
enum MyUnion {
    Int(i32),
    Float(f32),
    Char(char),
}
```

这样在实际使用时，可以通过`match`语句来根据`tag`值取用对应的数据：

```rust
match my_union {
    MyUnion::Int(i) => println!("int: {}", i),
    MyUnion::Float(f) => println!("float: {}", f),
    MyUnion::Char(c) => println!("char: {}", c),
}
```

注意，这里`match`的用法相当于C语言里的`switch`，
但是Rust把`tag`隐藏在`enum`的定义中，无法直接获取其`tag`值。

类似`if`的用法可以这么写：

```rust
if let MyUnion::Int(i) = my_union {
    println!("int: {}", i);
} else if let MyUnion::Float(f) = my_union {
    println!("float: {}", f);
} else if let MyUnion::Char(c) = my_union {
    println!("char: {}", c);
}
```

或者只需要判断单个case的时候：

```rust
if matches!(my_union, MyUnion::Int(_)) {
    // ...
}
```

### Auto语言

Auto语言既要兼容C，也要兼容Rust，
因此既要提供原始的`union`，
也需要提供`tagged union`，
并且最好还能保留直接获取tag的途径。

在Auto中，原始的`union`也使用和C差不多的语法：

```auto
union MyUnion {
  i int
  f float
  c char
}
```

Tagged-Union则选用`tag`关键字：

```auto
tag MyTag {
  Int int
  Float float
  Char char
}

这个定义相当于C里用`enum`定义Tag类型，再用`struct`+`union`定义数据类型：

这里前面的`Int`，`Float`和`Char`是tag名称，相当于C里的：

```c
typedef enum {
    Int,
    Float,
    Char,
} MyTagKind;
```

后面的`int`，`float`和`char`是实际的数据类型，相当于：

```c
typedef struct {
    MyTagKind tag;
    union {
        int Int;
        float Float;
        char Char;
    } as;
} MyTag;
```

Auto语言的`is`关键字相当于C的`switch`或Rust的`match`：

```auto
is my_tag_data {
    Int(i) => println!("int: {}", i),
    Float(f) => println!("float: {}", f),
    Char(c) => println!("char: {}", c),
}
```

可以直接获取`tag`值：

```auto
let t = my_tag_data.tag;
```

这里`t`的类型是`enum MyTag.Tag`，编译器会自动添加该枚举类型。

如果需要强制按照某种类型取出数据，可以使用`as`关键字：

```auto
sys {
    let i = my_tag_data.as.Int;
}
```

注意，这里的代码是不安全的，应当包含在`sys`框中。
