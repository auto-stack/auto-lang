import time

def main():
    x = type(42).__name__
    y = time.sleep(100 / 1000)
    t = time.time()
    print(x)

if __name__ == "__main__":
    main()
