# Statement Tests

Tests for Store, Body, Node, Fn, Param to_atom() output

## Store - Let

let x = 42

---

let(name("x"), type(int), expr(int(42)))

## Store - Mut

mut x = 42

---

mut(name("x"), type(int), expr(int(42)))

## Store - Const

const MAX = 100

---

const(name("MAX"), type(int), expr(int(100)))

## Store - With Type Annotation

let x: int = 42

---

let(name("x"), type(int), expr(int(42)))

## Store - Complex Expression

let result = add(1, 2) * 3

---

let(name("result"), type(int), expr(binary("*", call(name("add")) { int(1) int(2) } int(3))))

## Body - Single Statement

{ x }

---

body {
    ident(x)
}

## Body - Multiple Statements

{ x; y; z }

---

body {
    ident(x)
    ident(y)
    ident(z)
}

## Body - With Expression

{ x + y }

---

body {
    binary("+", ident(x), ident(y))
}

## Node - Simple Node

div(id="app") {
    p("Hello")
}

---

node(name("div"), id("app")) {
    call(name("p")) {
        str("Hello")
    }
}

## Node - With Multiple Properties

div(id="app", class="container") {
    p("Hello")
}

---

node(name("div"), id("app"), class("container")) {
    call(name("p")) {
        str("Hello")
    }
}

## Node - Nested Children

div(id="app") {
    div(class="header") {
        h1("Title")
    }
    div(class="content") {
        p("Content")
    }
}

---

node(name("div"), id("app")) {
    node(name("div"), class("header")) {
        call(name("h1")) {
            str("Title")
        }
    }
    node(name("div"), class("content")) {
        call(name("p")) {
            str("Content")
        }
    }
}

## Fn - Simple Function

fn add(x int, y int) int {
    x + y
}

---

fn(name("add")) {
    param(name("x"), type(int))
    param(name("y"), type(int))
    return(type(int))
    body {
        binary("+", ident(x), ident(y))
    }
}

## Fn - No Parameters

fn get_value() int {
    42
}

---

fn(name("get_value")) {
    return(type(int))
    body {
        int(42)
    }
}

## Fn - No Return Type

fn print_value(x int) {
    print(x)
}

---

fn(name("print_value")) {
    param(name("x"), type(int))
    body {
        call(name("print")) {
            ident(x)
        }
    }
}

## Fn - C Function Declaration

fn.c printf(format str, ...) int

---

fn.c(name("printf")) {
    param(name("format"), type(str))
    param(name("..."), type(void))
    return(type(int))
}

## Fn - Method

fn Point.new(x int, y int) Point {
    Point { x, y }
}

---

fn(name("Point.new")) {
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

## Param - Without Default

x: int

---

param(name("x"), type(int))

## Param - With Default

x: int = 0

---

param(name("x"), type(int), default(int(0)))

## Param - Multiple Types

x: int | str

---

param(name("x"), type(int), type(str))
