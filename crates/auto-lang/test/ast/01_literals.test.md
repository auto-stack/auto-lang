## int

41

---

int(41)

## float

42.3

---

float(42.3)


## str

"hello"

---

str("hello")


## fstr

`1 + 2 = ${1 + 2}`

---

fstr { str("1 + 2 = ") binary("+", int(1), int(2)) }
