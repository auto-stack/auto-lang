from typing import Protocol

class Flyer(Protocol):
    def fly(self): ...

class Pigeon:

    def fly(self):
        print("Flap")

def main():
    p = Pigeon()
    p.fly()

if __name__ == "__main__":
    main()
