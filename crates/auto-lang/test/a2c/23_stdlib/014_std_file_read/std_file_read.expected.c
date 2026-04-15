#include "std_file_read.h"

int main(void) {
    unknown f = File.open("test.txt");
    unknown ch = f.read_char();
    char* buf = "   ";
    unknown n = f.read_buf(buf, 3);
    f.close();
    return 0;
}
