def main():
    x = 10
    y = (x if x is not None else 0)
    print(y)

    z = None
    w = (z if z is not None else 42)
    print(w)

if __name__ == "__main__":
    main()
