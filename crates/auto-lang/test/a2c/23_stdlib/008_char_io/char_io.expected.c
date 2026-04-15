#include "char_io.h"

int main(void) {
    unknown file = open_write("char_test.txt");
    file.putc(65);
    file.putc(66);
    file.putc(67);
    file.close();

    unknown file2 = open_write("string_test.txt");
    file2.puts("Hello from puts!");
    file2.close();
    return 0;
}
