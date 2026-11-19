# Independent native-Python reference for protocol_1107.at (no ACE involved).
l = []
l.append(11)
l.append(22)
l.append(33)
n = len(repr(12345))
print(n)
print(l.count(22))
print(round(2.5, ndigits=0))
total = 0
for _ in range(3):
    c = l.count(22)
    if c == 1:
        total = total + 1
    total = total + 1
print(total)