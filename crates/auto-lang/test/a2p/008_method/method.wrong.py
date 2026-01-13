class Point:
    def __init__(self, x: int, y: int):
        self.x = x
        self.y = y


    def modulus(self):
        return self.x * self.x + self.y * self.y

def main():
    print("Method defined")

if __name__ == "__main__":
    main()
