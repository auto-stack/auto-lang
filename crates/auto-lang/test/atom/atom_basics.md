# Auto-Atom Basic Tests

This file contains basic tests for auto-atom functionality.

## Test Empty content

---

root {}

## Test One Pair

name: "test"

---

root {name: "test"}

## Test Two Pairs

name: "test"
value: 42

---

root {name: "test"; value: 42}

## Test Node with Multiple Properties

data {
  name: "example"
  age: 25
  active: true
  tags: "test"
}

---

root {data {name: "example"; age: 25; active: true; tags: "test"}}

## Test Array

[1, 2, 3]

---

[1, 2, 3]

## Test Obj

{
  greeting: "Hello, World!"
  who: "World"
}

---

{greeting: "Hello, World!", who: "World"}
