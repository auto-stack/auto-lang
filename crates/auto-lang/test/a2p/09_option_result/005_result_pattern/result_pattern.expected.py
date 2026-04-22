def main():
    res = 42
    match res:
        case OkCase(v):
            print(v)
        case ErrCase(e):
            print(e)

if __name__ == "__main__":
    main()
