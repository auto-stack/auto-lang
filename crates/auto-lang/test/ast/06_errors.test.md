## undefined variable with suggestion

let myVariable = 42
myVaraible

---

Error: auto_name_E0201

  × undefined variable
  help: Variable 'myVaraible' is not defined in this scope. Did you mean
        'myVariable'?

## undefined variable without close match

let x = 1
yz

---

Error: auto_name_E0201

  × undefined variable
  help: Variable 'yz' is not defined in this scope

## multiple similar variables

let counter = 0
let counter2 = 1
let counter3 = 2
cuonter

---

Error: auto_name_E0201

  × undefined variable
  help: Variable 'cuonter' is not defined in this scope. Did you mean
        'counter'?

## case sensitivity test

let UserName = "alice"
username

---

Error: auto_name_E0201

  × undefined variable
  help: Variable 'username' is not defined in this scope. Did you mean
        'UserName'?
