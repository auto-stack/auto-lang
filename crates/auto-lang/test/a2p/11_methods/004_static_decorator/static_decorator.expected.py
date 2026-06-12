@dataclass
class Math:

    @staticmethod
    def add(a: int, b: int) -> int:
        return a + b

def main():
    result = Math.add(3, 4)
    print(result)

if __name__ == "__main__":
    main()
