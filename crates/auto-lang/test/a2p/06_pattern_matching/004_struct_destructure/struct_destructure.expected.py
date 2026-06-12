from dataclasses import dataclass

@dataclass
class Point:
    x: int
    y: int

def main():
    p = Point(x=3, y=4)
    match p:
        case Point(x, y):
            print(x)

if __name__ == "__main__":
    main()
