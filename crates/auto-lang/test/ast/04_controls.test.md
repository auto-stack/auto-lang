## if control

if x > 1 {
    print("greater than 1")
} else {
    print("less than or equal to 1")
}

---

if { bina(>, x, 1) { call print ("greater than 1") }; else { call print ("less than or equal to 1") } }

## for control

for i in 0..10 {
    print(i)
}

---

for in(i, range(start(0), end(10), eq(false))) { call print (i) }

## for control

for var x = 0; x < 10 {
    print(x)
    x = x + 1
}

---

for (var(x, int, 0), bina(<, x, 10)) { call print (x); asn x bina(+, x, 1) }
