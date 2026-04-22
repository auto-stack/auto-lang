def get_name() -> str:
    x = "hello"
    y = x
    return y

def main():
    name = get_name()
    print(name)

if __name__ == "__main__":
    main()
