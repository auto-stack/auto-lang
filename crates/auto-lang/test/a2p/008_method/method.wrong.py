class Point:
    def __init__(self, x: Any, y: Any):
        self.x = x
        self.y = y

    def modulus(self):
        self.x * self.x + self.y * self.y

def main():
    print("Method defined")

if __name__ == "__main__":
    main()
