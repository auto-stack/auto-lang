## Function Decl & Body

fn add(a int, b int) int {
    a + b
}

---

fn add (param(a, int), param(b, int)) ret(int) {
    expr.bina("+", a, b)
}

## C Function Decl

fn.c square(n double) double

---

fn.c square (param(n, double))

## Function Call

add(1, 2)

---

call add {
    int(1)
    int(2)
}