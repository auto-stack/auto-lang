#include "advanced_io.h"

int main(void) {
    unknown file = open_write("seek_test.txt");

    file.puts("Hello");

    file.seek(0, 0);

    unknown pos = file.tell();

    file.rewind();

    file.close();
    return 0;
}
