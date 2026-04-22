def add(a: int, b: int) -> int:
    return a + b

def greet(name: str) -> str:
    return name

def main():
    result = add(5, 3)
    greeting = greet("world")
    print(result)
    print(greeting)

if __name__ == "__main__":
    main()
