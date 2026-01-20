#include "std_file_read.h"

int main(void) {
    struct File f = File.open("test.txt");
    int ch = File_ReadChar(&f);
    char* buf = "   ";
    int n = File_ReadBuf(&f, buf, 3);
    File_Close(&f);
    return 0;
}
