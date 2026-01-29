## unary expr

-1 

---

una(-, 1)


## binary expr

1 + 2

---

bina(+, 1, 2)


## index expr

a[1]

---

index(a, 1)


## slice expr

arr[1..10]

---

slice(arr, 1, 10)

## slice expr - with step

arr[0..10..2]

---

slice(arr, 0, 10, 2)

## array literal

[1, 2, 3]

---

array(1, 2, 3)

## object literal

{name: "John", age: 30}

---

obj { pair(name, "John"), pair(age, 30) }

## closure

(x, y) => x + y

---

|x y| bina(+, x, y)
