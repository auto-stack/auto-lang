# Advanced Control Flow Tests

Tests for If, Branch, For, Iter, Break, Range, Is, IsBranch to_atom() output

## If - Simple

if x > 0 {
    print("positive")
}

---

if {
    branch(binary(">", ident(x), int(0))) {
        call(name("print")) {
            str("positive")
        }
    }
}

## If - If-Else

if x > 0 {
    print("positive")
} else {
    print("non-positive")
}

---

if {
    branch(binary(">", ident(x), int(0))) {
        call(name("print")) {
            str("positive")
        }
    }
    else {
        call(name("print")) {
            str("non-positive")
        }
    }
}

## If - Multiple Else-If

if x > 0 {
    print("positive")
} else if x < 0 {
    print("negative")
} else {
    print("zero")
}

---

if {
    branch(binary(">", ident(x), int(0))) {
        call(name("print")) {
            str("positive")
        }
    }
    branch(binary("<", ident(x), int(0))) {
        call(name("print")) {
            str("negative")
        }
    }
    else {
        call(name("print")) {
            str("zero")
        }
    }
}

## Branch - Simple Branch

x > 0 {
    print("positive")
}

---

branch(binary(">", ident(x), int(0))) {
    call(name("print")) {
        str("positive")
    }
}

## For - Range Loop

for i in 0..10 {
    print(i)
}

---

for(iter(name("i"), range(int(0), int(10)))) {
    call(name("print")) {
        ident(i)
    }
}

## For - Inclusive Range

for i in 0..=10 {
    print(i)
}

---

for(iter(name("i"), range(int(0), int(10), eq(true)))) {
    call(name("print")) {
        ident(i)
    }
}

## For - With Index

for (i, item) in items {
    print(i, item)
}

---

for(iter(name("i", "item"), ident(items))) {
    call(name("print")) {
        ident(i)
        ident(item)
    }
}

## For - Infinite Loop

for ever {
    do_work()
}

---

for(iter(ever)) {
    call(name("do_work"))
}

## Break - Simple

break

---

break

## Range - Exclusive

0..10

---

range(start(int(0)), end(int(10)), eq(false))

## Range - Inclusive

0..=10

---

range(start(int(0)), end(int(10)), eq(true))

## Is - Pattern Matching

is x {
    1 => print("one")
    2 => print("two")
    _ => print("other")
}

---

is(ident(x)) {
    eq(int(1)) {
        call(name("print")) {
            str("one")
        }
    }
    eq(int(2)) {
        call(name("print")) {
            str("two")
        }
    }
    else {
        call(name("print")) {
            str("other")
        }
    }
}

## Is - With If Branch

is value {
    if x > 0 => print("positive")
    if x < 0 => print("negative")
    _ => print("zero")
}

---

is(ident(value)) {
    if(binary(">", ident(x), int(0))) {
        call(name("print")) {
            str("positive")
        }
    }
    if(binary("<", ident(x), int(0))) {
        call(name("print")) {
            str("negative")
        }
    }
    else {
        call(name("print")) {
            str("zero")
        }
    }
}

## IsBranch - Eq Branch

1 => print("one")

---

eq(int(1)) {
    call(name("print")) {
        str("one")
    }
}

## IsBranch - If Branch

if x > 0 => print("positive")

---

if(binary(">", ident(x), int(0))) {
    call(name("print")) {
        str("positive")
    }
}

## IsBranch - Else Branch

_ => print("other")

---

else {
    call(name("print")) {
        str("other")
    }
}
