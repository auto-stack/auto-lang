## Integer Literal

42

---

expr.int(value: 42)

## Identifier

x

---

expr.ident(name: x)

## Binary Operation

1 + 2

---

expr.binary(op: +) { expr.int(value: 1), expr.int(value: 2) }

## Array Literal

[1, 2, 3]

---

expr.array(count: 3) { expr.int(value: 1), expr.int(value: 2), expr.int(value: 3) }

## Variable Declaration

var x = 42

---

stmt.store(name: x) { expr.int(value: 42) }

## For Loop

for i in 0..3 { i }

---

stmt.for(var: i, iter: expr.binary(op: ..) { expr.int(value: 0), expr.int(value: 3) }) { expr.ident(name: i) }

## Block Statement

{ var x = 1; x + 2 }

---

stmt.block(count: 2) { stmt.store(name: x) { expr.int(value: 1) }, stmt.expr() { expr.binary(op: +) { expr.ident(name: x), expr.int(value: 2) } } }

## Multiple Statements

var x = 42
x
x + 1

---

Code(count: 3) { stmt.store(name: x) { expr.int(value: 42) }, stmt.expr() { expr.ident(name: x) }, stmt.expr() { expr.binary(op: +) { expr.ident(name: x), expr.int(value: 1) } } }

## Range Expression

0..10

---

expr.binary(op: ..) { expr.int(value: 0), expr.int(value: 10) }

## Function Call

print(42)

---

expr.call(callee: expr.ident(name: print), args: 1) { expr.int(value: 42) }

## String Literal

"hello"

---

expr.str(value: hello)

## Boolean Literal

true

---

expr.bool(value: true)

## Unary Operation

-42

---

expr.unary(op: -, expr.int(value: 42))

## If Expression

if true { 1 } else { 0 }

---

expr.if(cond: expr.bool(value: true), then: expr.int(value: 1), else: expr.int(value: 0))

## Array Index

arr[0]

---

expr.index(array: expr.ident(name: arr), index: expr.int(value: 0))

## Object Literal

{ a: 1, b: 2 }

---

expr.object(count: 2) { a: expr.int(value: 1), b: expr.int(value: 2) }

## While Loop

while x < 10 { x = x + 1 }

---

stmt.while(cond: expr.binary(op: <) { expr.ident(name: x), expr.int(value: 10) }, body: stmt.block(count: 1) { stmt.store(name: x) { expr.binary(op: +) { expr.ident(name: x), expr.int(value: 1) } } } )

## If Statement

if x > 0 { print(x) }

---

stmt.if(cond: expr.binary(op: >) { expr.ident(name: x), expr.int(value: 0) }, then: stmt.expr() { expr.call(callee: expr.ident(name: print), args: 1) { expr.ident(name: x) } } )
