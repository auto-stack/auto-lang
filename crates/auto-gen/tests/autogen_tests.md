# Auto-Gen Test Cases

Test cases for auto-gen code generation functionality. Each test has three sections:
1. Data (Auto code)
2. Template (Auto template)
3. Expected output

---

## Test 1: Simple Variable

name: "World"

---

Hello, $name!

---

Hello, World!

---

## Test 2: Multiple Variables

greeting: "Hello"
target: "World"

---

$greeting, $target!

---

Hello, World!

---

## Test 3: Integer Variable

count: 42

---

The answer is $count.

---

The answer is 42.

---

## Test 4: Boolean Variable

is_ready: true

---

Status: $is_ready

---

Status: true

---

## Test 5: Simple Array

items: [1, 2, 3]

---

$ for i in items {
[$i, ${i*2}, ${i*3}]
$ }

---

[1, 2, 3]
[2, 4, 6]
[3, 6, 9]

---

## Test 6: Array with Fields

products: [
    { name: "A", price: 10 }
    { name: "B", price: 20 }
]

---
{
$ for p in products {
    ${p.name}: ${p.price}
$ }
}

---

{
    A: 10
    B: 20
}

---

## Test 7: Nested Fields

user: {
  name: "John"
  age: 30
}

---

Name: ${user.name}, Age: ${user.age}

---

Name: John, Age: 30

---

## Test 8: Array Iteration

values: [10, 20, 30]

---

$ for i, v in values {
Index $i: $v
$ }

---

Index 0: 10
Index 1: 20
Index 2: 30

---

## Test 9: Conditional Text

debug: true

---

$ if debug {
#define DEBUG_MODE ON
$ }

---

#define DEBUG_MODE ON

---

## Test 10: Auto with variables

let name = "World"

first: "Hello"
second: name // here name is evaluated

---

$first $second!

---

Hello World!
---
