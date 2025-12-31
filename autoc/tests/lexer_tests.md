## Basic Lexer

(123)

---

<(><int:123><)>

## String Literal

"Hello, World!"

---

<str:Hello, World!>

## Simple String

"Hello"

---

<str:Hello>

## Range Expression

1..5

---

<int:1><..><int:5>

## Pair/Object Literal

a: 3
b: 4

---

<ident:a><:><int:3><nl><ident:b><:><int:4>

## F-String with Variable

f"hello $you again"

---

<ident:f><fstrs><fstrp:hello ><$><ident:you><fstrp: again><fstre>

## F-String with Expression

f"hello ${2 + 1} again"

---

<ident:f><fstrs><fstrp:hello ><$><{><int:2><+><int:1><}><fstrp: again><fstre>

## Tick String with Variable

`hello $you again`

---

<fstrs><fstrp:hello ><$><ident:you><fstrp: again><fstre>

## Comments

// this is a comment
/* this is a multi-line comment */

---

<//><comment:...><nl></*><comment:...><*/>

## F-String Multiple Expressions

`hello $name ${age}`

---

<fstrs><fstrp:hello ><$><ident:name><fstrp: ><$><{><ident:age><}><fstre>

## F-String (f prefix)

f"hello $name"

---

<ident:f><fstrs><fstrp:hello ><$><ident:name><fstre>

## Unsigned Integer

125u

---

<uint:125>

## U8 Integer

125u8

---

<u8:125>

## I8 Integer

41i8

---

<i8:41>

## Path Expression

a.b.c: x, y

---

<ident:a><.><ident:b><.><ident:c><:><ident:x><,><ident:y>

## Char Literal

'a'

---

<'a'>

## F-String Lexer Test

`${mid(){}}`

---

<fstrs><fstrp:><$><{><ident:mid><(><)><{><}><}><fstre>

## Arrow Notation

5 -> 7

---

<int:5><->><int:7>

## On Block

on {
    5 -> 7
    6 -> 8 : 10
}

---

<on><{><nl><int:5><->><int:7><nl><int:6><->><int:8><:><int:10><nl><}>

## On in Function

fn on(ev str) {}

---

<fn><ident:on><(><ident:ev><ident:str><)><{><}>

## Use C Statement

use c <stdio.h>

---

<use><ident:c><lt><ident:stdio><.><ident:h><gt>

## C String Literal

c"hello"

---

<cstr:hello>

## At Symbol

@int

---

<@><ident:int>

## Minus One

a-1

---

<ident:a><-><int:1>

## Equality

==

---

<==>

## Not Equal

!=

---

<!=>

## Greater Than

>

---

<>>

## Less Than

<

---

<<>

## Greater or Equal

>=

---

<>=>>

## Less or Equal

<=

---

<<=>

## Add Equal

+=

-->

<+=>>

## Subtract Equal

-=

-->

<-=>>

## Multiply Equal

*=

-->

<*=>>

## Divide Equal

/=>

-->

</=>>

## Double Arrow

=>

-->

<=>>
