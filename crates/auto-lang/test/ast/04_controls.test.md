## if control

if x > 1 {
    print("greater than 1")
} else {
    print("less than or equal to 1")
}

---

if {
    bina('>', x, int(1)) {
        call print ("greater than 1")
    }

    else {
        call print ("less than or equal to 1")
    }
}

## for control

for i in range(0, 10) {
    print(i)
}

---

for in(i, range(int(0), int(10))) {
    call print (i)
}

## while control

while x < 10 {
    print(x)
    x = x + 1
}

---

while bina('<', x, int(10)) {
    call print (x)
    asn x (bina('+', x, int(1)))
}
