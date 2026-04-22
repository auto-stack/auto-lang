def double(x: int) -> int:
    return x * 2

def main():
    result = double(double(3))
    print(result)

if __name__ == "__main__":
    main()
