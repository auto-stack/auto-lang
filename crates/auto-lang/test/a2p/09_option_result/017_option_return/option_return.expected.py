from typing import Optional

def maybe_val() -> Optional[int]:
    return 42

def main():
    v = maybe_val()
    print(v)

if __name__ == "__main__":
    main()
