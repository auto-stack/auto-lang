@dataclass
class Holder:
    val: int

    def get(self) -> int:
        return self.val

def main():
    b = Holder(val=10)
    v = b.get()
    print(v)

if __name__ == "__main__":
    main()
