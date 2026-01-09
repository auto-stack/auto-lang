# Events and Calls Tests

Tests for Call, Args, Arg, OnEvents, Event, Arrow, CondArrow to_atom() output

## Call - Simple

print("hello")

---

call(name("print")) {
    str("hello")
}

## Call - Multiple Arguments

add(1, 2, 3)

---

call(name("add")) {
    int(1)
    int(2)
    int(3)
}

## Call - With Named Arguments

printf(format="Hello %s", name="World")

---

call(name("printf")) {
    pair(name("format"), str("Hello %s"))
    pair(name("name"), str("World"))
}

## Call - Mixed Arguments

add(1, 2, z: 3)

---

call(name("add")) {
    int(1)
    int(2)
    pair(name("z"), int(3))
}

## Call - Nested Calls

add(mul(2, 3), 4)

---

call(name("add")) {
    call(name("mul")) {
        int(2)
        int(3)
    }
    int(4)
}

## Call - Method Call

obj.get_value()

---

call(name("obj.get_value"))

## Call - With Return Type Annotation

let result: int = add(1, 2)

---

let(name("result"), type(int), expr(call(name("add")) { int(1) int(2) }))

## Args - Empty

func()

---

args()

## Args - Positional Only

func(1, 2, 3)

---

args(int(1), int(2), int(3))

## Args - Named Only

func(x: 1, y: 2)

---

args(pair(name("x"), int(1)), pair(name("y"), int(2)))

## Args - Mixed

func(1, 2, z: 3)

---

args(int(1), int(2), pair(name("z"), int(3)))

## Arg - Positional

1

---

int(1)

## Arg - Named

x: 1

---

pair(name("x"), int(1))

## Arg - Pair Expression

user.name: "John"

---

pair(name("user.name"), str("John"))

## OnEvents - Simple

on click {
    print("clicked")
}

---

on {
    arrow(from(event("click")), to(call(name("print")) { str("clicked") }))
}

## OnEvents - Multiple Events

on click {
    print("clicked")
}
on hover {
    print("hovered")
}

---

on {
    arrow(from(event("click")), to(call(name("print")) { str("clicked") }))
    arrow(from(event("hover")), to(call(name("print")) { str("hovered") }))
}

## Event - Simple

click

---

event("click")

## Event - With Properties

keydown(key="Enter")

---

event("keydown", key("Enter"))

## Arrow - Simple

click => print("clicked")

---

arrow(from(event("click")), to(call(name("print")) { str("clicked") }))

## Arrow - With With Clause

click => print("clicked") with console

---

arrow(from(event("click")), to(call(name("print")) { str("clicked") }), with(ident(console)))

## Arrow - To Nested Action

click => {
    log("clicked")
    print("clicked")
}

---

arrow(from(event("click")), to(body {
    call(name("log")) { str("clicked") }
    call(name("print")) { str("clicked") }
}))

## CondArrow - Simple

if active => click => print("clicked")

---

cond-arrow(from(ident(active)), cond(event("click"))) {
    arrow(to(call(name("print")) { str("clicked") }))
}

## CondArrow - Multiple Branches

if state == "ready" => click => start()
if state == "running" => click => stop()

---

cond-arrow(from(ident(state))) {
    eq(str("ready")) {
        arrow(from(event("click")), to(call(name("start"))))
    }
    eq(str("running")) {
        arrow(from(event("click")), to(call(name("stop"))))
    }
}

## CondArrow - With Else

if logged_in => click => dashboard()
else => click => login()

---

cond-arrow(from(ident(logged_in))) {
    eq(bool(true)) {
        arrow(from(event("click")), to(call(name("dashboard"))))
    }
    else {
        arrow(from(event("click")), to(call(name("login"))))
    }
}
