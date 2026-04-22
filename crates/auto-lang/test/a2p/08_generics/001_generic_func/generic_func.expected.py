def identity(x):
    return x

def main():
    a = identity(42)
    b = identity("hello")
    print(a)
    print(b)

if __name__ == "__main__":
    main()
