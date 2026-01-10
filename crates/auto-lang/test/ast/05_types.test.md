# Type System Tests

Tests for Type, Key, Pair, Member, TypeDecl to_atom() output

## Type - Int

let a int = 10

---

let(a, int, 10)

## Type - Float

var x float = 3.14

---

var(x, float, 3.14)

## Type - Bool

var z bool = true

---

var(z, bool, true)

## Type - Str

let s str = "hello"

---

let(s, str, "hello")

## Type - Void

fn say() void {
    print("Hello, World!")
}

---

fn say () void { call print ("Hello, World!") }

## Type - Pointer

let p *int = 10.ptr

---

let(p, ptr(int), bina(10, ptr))

## Type - Array

var arr [3]int = [1, 2, 3]

---

var(arr, array(int, 3), array(1, 2, 3))

## Type - User Defined

type Point {
    x int
    y int
}

fn new_point(x int, y int) Point {
    Point(x, y)
}

---

type Point { member(x, int); member(y, int) }; (nl (count 1)); fn new_point ((x, int), (y, int)) Point { node Point (x, y) }

## Key - Named

name: "me"

---

pair(name, "me")

## Key - Integer

42: 5

---

pair(42, 5)

## Key - Boolean

true: "good"

---

pair(true, "good")

## Key - String

"key": "value"

---

pair("key", "value")

## Pair - Simple

name: "value"

---

pair(name, "value")

## TypeDecl - Simple Struct

type Point {
    x int = 0
    y int
}

---

type Point { member(x, int, 0); member(y, int) }

## TypeDecl - With Methods

type Point {
    x int
    y int
    
    fn new(x int, y int) Point {
        Point(x, y)
    }
}

---

type Point { member(x, int); member(y, int); fn new ((x, int), (y, int)) Point { node Point (x, y) } }
