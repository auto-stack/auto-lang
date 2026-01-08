# Type System Tests

Tests for Type, Key, Pair, Member, TypeDecl to_atom() output

## Type - Int

int

---

int

## Type - Float

float

---

float

## Type - Bool

bool

---

bool

## Type - Str

str

---

str

## Type - Void

void

---

void

## Type - Pointer

*int

---

ptr(int)

## Type - Array

[int; 10]

---

array(int, 10)

## Type - User Defined

type Point

---

Point

## Key - Named

name

---

name("name")

## Key - Integer

42

---

int(42)

## Key - Boolean

true

---

bool(true)

## Key - String

"key"

---

str("key")

## Pair - Simple

name: value

---

pair(name("name"), ident(value))

## Pair - Nested

user.name: first_name

---

pair(name("user.name"), ident(first_name))

## Member - Without Default

x: int

---

member(name("x"), type(int))

## Member - With Default Value

x: int = 42

---

member(name("x"), type(int), value(int(42)))

## TypeDecl - Simple Struct

type Point {
    x: int
    y: int
}

---

type-decl(name("Point")) {
    member(name("x"), type(int))
    member(name("y"), type(int))
}

## TypeDecl - With Methods

type Point {
    x: int
    y: int
    
    fn new(x int, y int) Point {
        Point { x, y }
    }
}

---

type-decl(name("Point")) {
    member(name("x"), type(int))
    member(name("y"), type(int))
    fn(name("new")) {
        param(name("x"), type(int))
        param(name("y"), type(int))
        return(type(Point))
        body {
            call(name("Point")) {
                ident(x)
                ident(y)
            }
        }
    }
}

## TypeDecl - Generic Type

type List[T] {
    head: T
    tail: *List[T]
}

---

type-decl(name("List"), has(T)) {
    member(name("head"), type(T))
    member(name("tail"), type(ptr(List(T))))
}
