def divide(a: int, b: int) -> int:
    if b == 0:
        return 0
    return a / b

def main():
    result = divide(10, 2)
    print(result)

if __name__ == "__main__":
    main()
