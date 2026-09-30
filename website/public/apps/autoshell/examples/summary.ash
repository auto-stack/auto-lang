var amounts = [120, 85, 210, 40]
var total = 0
var large = 0

for amount in amounts {
    total = total + amount
    if amount >= 100 {
        large = large + 1
    }
}

print("items: " + amounts.len().str())
print("total: " + total.str())
print("amounts >= 100: " + large.str())
