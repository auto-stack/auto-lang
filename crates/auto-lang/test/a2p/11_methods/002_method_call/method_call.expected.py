@dataclass
class Counter:
    count: int

    def inc(self):
        self.count = self.count + 1

    def get(self) -> int:
        return self.count

def main():
    c = Counter(count=0)
    c.inc()
    c.inc()
    print(c.get())

if __name__ == "__main__":
    main()
