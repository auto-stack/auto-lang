from dataclasses import dataclass

@dataclass
class Person:
    name: str
    age: int
    height: float
    active: bool

def main():
    print("Person struct defined")

if __name__ == "__main__":
    main()
