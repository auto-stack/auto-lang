#include "std_readline.h"

int main(void) {
    unknown f = File.open("Cargo.toml");

    unknown s = f.read_line();
    printf("%d\n", s);

    f.close();
    return 0;
}
