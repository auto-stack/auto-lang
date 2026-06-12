@dataclass
class Point:
    x: int
    y: int

    def modulus(self) -> int:
        return self.x * self.x + self.y * self.y

def main():
    print("Method defined")

if __name__ == "__main__":
    main()
