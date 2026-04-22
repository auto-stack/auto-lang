from typing import Protocol

class Flyer(Protocol):
    def fly(self): ...

def main():
    print("Spec defined")

if __name__ == "__main__":
    main()
