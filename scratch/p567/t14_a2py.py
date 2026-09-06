from builtins import open

def main():
    with open("out.txt", "w") as f:
        f.write("hi")

if __name__ == "__main__":
    main()
