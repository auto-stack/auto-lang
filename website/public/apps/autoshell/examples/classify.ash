fn bucket(n) {
    if n >= 80 { return "high" }
    return "low"
}

var values = [95, 82, 61]
for v in values {
    print(v.str() + ":" + bucket(v))
}
