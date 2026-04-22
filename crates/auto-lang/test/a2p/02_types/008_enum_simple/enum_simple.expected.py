from enum import Enum, auto

class Direction(Enum):
    North = auto()
    South = auto()
    East = auto()
    West = auto()

def main():
    print("Enum defined")

if __name__ == "__main__":
    main()
