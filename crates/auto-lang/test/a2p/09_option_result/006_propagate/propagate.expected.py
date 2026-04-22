def test_propagate() -> int:
    x = 10
    y = x
    return y

def main():
    result = test_propagate()
    print(result)

if __name__ == "__main__":
    main()
