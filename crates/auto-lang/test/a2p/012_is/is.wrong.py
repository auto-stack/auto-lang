def main():
    x = 10
    match x:
        case 0:
            print("X is ZERO")
        case 1:
            print("X is ONE")
        case _:
            print("X is Large")

if __name__ == "__main__":
    main()
