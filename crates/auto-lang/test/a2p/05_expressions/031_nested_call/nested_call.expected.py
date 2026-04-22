def add(a: int, b: int) -> int:
    return a + b

def main():
    result = add(add(1, 2), add(3, 4))
    print(result)

if __name__ == "__main__":
    main()
