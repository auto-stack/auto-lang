@dataclass
class Counter:
    count: int

    def increment(self):
        self.count = self.count + 1

def main():
    print("Type with methods defined")

if __name__ == "__main__":
    main()
