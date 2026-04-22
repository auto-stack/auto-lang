def main():
    x = 42
    match x:
        case SomeCase(v):
            print(v)
        case None:
            print("is none")

if __name__ == "__main__":
    main()
