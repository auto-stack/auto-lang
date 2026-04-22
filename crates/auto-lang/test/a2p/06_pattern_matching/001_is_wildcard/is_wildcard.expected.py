def main():
    x = 5
    match x:
        case 0:
            print("zero")
        case 1:
            print("one")
        case _:
            print("other")

if __name__ == "__main__":
    main()
