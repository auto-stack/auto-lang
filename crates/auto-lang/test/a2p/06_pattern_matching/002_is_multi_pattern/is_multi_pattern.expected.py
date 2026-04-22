def main():
    x = 3
    match x:
        case 1:
            print("one")
        case 2:
            print("two")
        case 3:
            print("three")
        case _:
            print("other")

if __name__ == "__main__":
    main()
