# Edge Cases and Complex Scenarios Tests

Tests for nested structures, complex expressions, and edge cases

## Nested Function Calls

let result = add(mul(2, 3), div(10, 2))

---

let(name("result"), type(int), expr(call(name("add")) {
    call(name("mul")) {
        int(2)
        int(3)
    }
    call(name("div")) {
        int(10)
        int(2)
    }
}))

## Deeply Nested If

if x > 0 {
    if y > 0 {
        if z > 0 {
            print("all positive")
        } else {
            print("x and y positive, z non-positive")
        }
    } else {
        print("x positive, y non-positive")
    }
} else {
    print("x non-positive")
}

---

if {
    branch(binary(">", ident(x), int(0))) {
        if {
            branch(binary(">", ident(y), int(0))) {
                if {
                    branch(binary(">", ident(z), int(0))) {
                        call(name("print")) {
                            str("all positive")
                        }
                    }
                    else {
                        call(name("print")) {
                            str("x and y positive, z non-positive")
                        }
                    }
                }
            }
            else {
                call(name("print")) {
                    str("x positive, y non-positive")
                }
            }
        }
    }
    else {
        call(name("print")) {
            str("x non-positive")
        }
    }
}

## Complex Object with Nested Pairs

config {
    name: "App"
    settings: {
        debug: true
        port: 8080
        hosts: ["localhost", "127.0.0.1"]
    }
}

---

object {
    pair(name("name"), str("App"))
    pair(name("settings"), object {
        pair(name("debug"), bool(true))
        pair(name("port"), int(8080))
        pair(name("hosts"), array(str("localhost"), str("127.0.0.1")))
    })
}

## Nested Node with Multiple Properties

div(id="app", class="container", style="width: 100%") {
    header(class="header") {
        h1(class="title") {
            text("Welcome")
        }
        nav(class="menu") {
            a(href="/home") {
                text("Home")
            }
            a(href="/about") {
                text("About")
            }
        }
    }
    main(class="content") {
        article {
            p("Content here")
        }
    }
}

---

node(name("div"), id("app"), class("container"), style("width: 100%")) {
    node(name("header"), class("header")) {
        node(name("h1"), class("title")) {
            call(name("text")) {
                str("Welcome")
            }
        }
        node(name("nav"), class("menu")) {
            node(name("a"), href("/home")) {
                call(name("text")) {
                    str("Home")
                }
            }
            node(name("a"), href("/about")) {
                call(name("text")) {
                    str("About")
                }
            }
        }
    }
    node(name("main"), class("content")) {
        node(name("article")) {
            call(name("p")) {
                str("Content here")
            }
        }
    }
}

## Complex For Loop with Nested Control Flow

for i in 0..10 {
    if i % 2 == 0 {
        print("even:", i)
    } else {
        print("odd:", i)
    }
    if i == 5 {
        break
    }
}

---

for(iter(name("i"), range(int(0), int(10)))) {
    if {
        branch(binary("==", binary("%", ident(i), int(2)), int(0))) {
            call(name("print")) {
                str("even:")
                ident(i)
            }
        }
        else {
            call(name("print")) {
                str("odd:")
                ident(i)
            }
        }
    }
    if {
        branch(binary("==", ident(i), int(5))) {
            break
        }
    }
}

## Is Pattern Matching with Nested Patterns

is value {
    Point(x, y) => print("Point:", x, y)
    Circle(radius, center) => print("Circle:", radius, center)
    _ => print("Other")
}

---

is(ident(value)) {
    eq(call(name("Point")) { ident(x) ident(y) }) {
        call(name("print")) {
            str("Point:")
            ident(x)
            ident(y)
        }
    }
    eq(call(name("Circle")) { ident(radius) ident(center) }) {
        call(name("print")) {
            str("Circle:")
            ident(radius)
            ident(center)
        }
    }
    else {
        call(name("print")) {
            str("Other")
        }
    }
}

## Function with Multiple Returns and Error Handling

fn divide(a int, b int) int | Error {
    if b == 0 {
        return Error("Division by zero")
    } else {
        return a / b
    }
}

---

fn(name("divide")) {
    param(name("a"), type(int))
    param(name("b"), type(int))
    return(type(int), type(Error))
    body {
        if {
            branch(binary("==", ident(b), int(0))) {
                call(name("return")) {
                    call(name("Error")) {
                        str("Division by zero")
                    }
                }
            }
            else {
                call(name("return")) {
                    binary("/", ident(a), ident(b))
                }
            }
        }
    }
}

## Complex Event Chain

on init => setup()
on ready => {
    load_data()
    render()
}
on error => {
    log("Error occurred")
    show_error()
}

---

on {
    arrow(from(event("init")), to(call(name("setup"))))
}
on {
    arrow(from(event("ready")), to(body {
        call(name("load_data"))
        call(name("render"))
    }))
}
on {
    arrow(from(event("error")), to(body {
        call(name("log")) {
            str("Error occurred")
        }
        call(name("show_error"))
    }))
}

## Generic TypeDecl with Multiple Type Parameters

type Result[T, E] {
    Ok(T)
    Err(E)
}

---

type-decl(name("Result"), has(T, E)) {
    field(name("Ok"), type(T))
    field(name("Err"), type(E))
}

## Union with Complex Field Types

union Value {
    Int(int)
    Float(float)
    String(str)
    Array([int])
    Object({str: int})
    Bool(bool)
    Null
}

---

union(name("Value")) {
    field(name("Int"), type(int))
    field(name("Float"), type(float))
    field(name("String"), type(str))
    field(name("Array"), type(array(int)))
    field(name("Object"), type(object(pair(name("str"), type(int)))))
    field(name("Bool"), type(bool))
    field(name("Null"), type(void))
}

## Empty Structures

type Empty {}

---

type-decl(name("Empty"))

fn empty() {
}

---

fn(name("empty")) {
    body
}

## Array Operations

let arr = [1, 2, 3]
let sliced = arr[1..2]
let indexed = arr[0]

---

let(name("arr"), type(array(int)), expr(array(int(1), int(2), int(3))))
let(name("sliced"), type(array(int)), expr(slice(ident(arr), int(1), int(2))))
let(name("indexed"), type(int), expr(index(ident(arr), int(0))))
