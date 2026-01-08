## unary expr

-1 

---

unary("-", int(1))


## binary expr

1 + 2

---

binary("+", int(1), int(2))


## index expr

a[1]

---

index(ident(a), int(1))


## slice expr

arr[1..10]

---

slice(ident(arr), int(1), int(10))

## slice expr - with step

arr[0..10..2]

---

slice(ident(arr), int(0), int(10), int(2))

## array literal

[1, 2, 3]

---

array(int(1), int(2), int(3))

## object literal

{name: "John", age: 30}

---

object {
    pair(name("name"), str("John"))
    pair(name("age"), int(30))
}

## lambda

fn(x, y) { x + y }

---

lambda {
    param(name("x"))
    param(name("y"))
    body {
        binary("+", ident(x), ident(y))
    }
}