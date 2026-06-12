import json

from dataclasses import dataclass

@dataclass
class Point:
    x: int
    y: int

def main():
    p = Point(1, 2)
    print(p.x)

if __name__ == "__main__":
    main()
