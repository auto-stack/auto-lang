def main():
    items = [1, 2, 3]
    items.push(4)
    last = items.pop()
    n = items.len()
    has = items.contains(2)
    name = "  hello  ".trim()
    parts = "a,b,c".split(",")
    upper = "hello".to_upper()
    lower = "HELLO".to_lower()
    yes = "hello".starts_with("he")
    print(n)

if __name__ == "__main__":
    main()
