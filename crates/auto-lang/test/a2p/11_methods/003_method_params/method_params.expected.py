@dataclass
class Calculator:

    def multiply(self, a: int, b: int) -> int:
        return a * b

def main():
    c = Calculator()
    print(c.multiply(6, 7))

if __name__ == "__main__":
    main()
