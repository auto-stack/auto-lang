from dataclasses import dataclass

@dataclass
class MyUnion:
    kind: str = ''
    i: int = 0
    f: float = 0.0

def main():
    print("Union defined")

if __name__ == "__main__":
    main()
