from dataclasses import dataclass

@dataclass
class Value:
    kind: str = ''
    i: int = 0
    s: str = ""
    d: float = 0.0

def main():
    v = Value(i=42)
    print("Union match")

if __name__ == "__main__":
    main()
