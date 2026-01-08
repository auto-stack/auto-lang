# Additional Expression Tests

More comprehensive expression tests for complete coverage

## Unary - Not

!true

---

unary("!", bool(true))

## Unary - Bitwise Not

~0xFF

---

unary("~", int(255))

## Unary - Dereference

*ptr

---

unary("*", ident(ptr))

## Unary - Reference

&value

---

unary("&", ident(value))

## Binary - Modulo

10 % 3

---

binary("%", int(10), int(3))

## Binary - Bitwise And

x & y

---

binary("&", ident(x), ident(y))

## Binary - Bitwise Or

x | y

---

binary("|", ident(x), ident(y))

## Binary - Bitwise Xor

x ^ y

---

binary("^", ident(x), ident(y))

## Binary - Left Shift

x << 2

---

binary("<<", ident(x), int(2))

## Binary - Right Shift

x >> 2

---

binary(">>", ident(x), int(2))

## Binary - Logical And

x && y

---

binary("&&", ident(x), ident(y))

## Binary - Logical Or

x || y

---

binary("||", ident(x), ident(y))

## Binary - Less Than or Equal

x <= 10

---

binary("<=", ident(x), int(10))

## Binary - Greater Than or Equal

x >= 10

---

binary(">=", ident(x), int(10))

## Binary - Not Equal

x != 10

---

binary("!=", ident(x), int(10))

## Binary - Equality

x == 10

---

binary("==", ident(x), int(10))

## Binary - Assignment

x = 42

---

binary("=", ident(x), int(42))

## Binary - Add Assignment

x += 10

---

binary("+=", ident(x), int(10))

## Binary - Range Expression

0..100

---

binary("..", int(0), int(100))

## Binary - Inclusive Range Expression

0..=100

---

binary("..=", int(0), int(100))

## Index - Multi-dimensional

matrix[2][3]

---

index(index(ident(matrix), int(2)), int(3))

## Index - With Expression

arr[index + 1]

---

index(ident(arr), binary("+", ident(index), int(1)))

## Slice - With Start Only

arr[1..]

---

slice(ident(arr), int(1))

## Slice - With End Only

arr[..10]

---

slice(ident(arr), end(int(10)))

## Slice - Full Slice

arr[..]

---

slice(ident(arr))

## Slice - With Step

arr[0..10..2]

---

slice(ident(arr), int(0), int(10), int(2))

## Array - Empty

[]

---

array()

## Array - Mixed Types

[1, "hello", true, 3.14]

---

array(int(1), str("hello"), bool(true), float(3.14)))

## Array - Nested Arrays

[[1, 2], [3, 4]]

---

array(array(int(1), int(2)), array(int(3), int(4)))

## Object - Empty

{}

---

object()

## Object - Nested Objects

{
    outer: {
        inner: "value"
    }
}

---

object {
    pair(name("outer"), object {
        pair(name("inner"), str("value"))
    })
}

## Object - Array Properties

{
    items: [1, 2, 3]
}

---

object {
    pair(name("items"), array(int(1), int(2), int(3)))
}

## Lambda - Closures

fn(x) { x * 2 }

---

lambda {
    param(name("x"))
    body {
        binary("*", ident(x), int(2))
    }
}

## Lambda - Closure Over Multiple Variables

fn(x, y, z) { x + y + z }

---

lambda {
    param(name("x"))
    param(name("y"))
    param(name("z"))
    body {
        binary("+", binary("+", ident(x), ident(y)), ident(z))
    }
}

## Lambda - With Return Type

fn(x int) int { x + 1 }

---

lambda {
    param(name("x"), type(int))
    return(type(int))
    body {
        binary("+", ident(x), int(1))
    }
}

## Complex Expression - Ternary-like

if condition { 1 } else { 0 }

---

if {
    branch(ident(condition)) {
        int(1)
    }
    else {
        int(0)
    }
}

## Complex Expression - Method Chaining

obj.method1().method2()

---

call(name("obj.method2")) {
    call(name("obj.method1"))
}

## Complex Expression - Field Access Chain

obj.field1.field2

---

binary(".", ident(obj), ident(field1), ident(field2))
