from dataclasses import dataclass

@dataclass
class Point:
    x: int
    y: int

@dataclass
class Rect:
    p1: Point
    p2: Point

def main():
    print("Nested struct defined")

if __name__ == "__main__":
    main()
