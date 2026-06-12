def main():
    items = [1, 2, 3]
    items.append(4)
    last = items.pop()
    n = len(items)
    has = 2 in items
    name = "  hello  ".strip()
    parts = "a,b,c".split(",")
    upper = "hello".upper()
    lower = "HELLO".lower()
    yes = "hello".startswith("he")
    print(n)

if __name__ == "__main__":
    main()
