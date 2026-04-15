#include "std_file.h"

int main(void) {
    unknown file = File.open("Cargo.toml");
    unknown s = file.read_text();
    printf("%d\n", s);
    file.close();
    return 0;
}
