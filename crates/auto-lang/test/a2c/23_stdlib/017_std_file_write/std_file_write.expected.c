#include "std_file_write.h"

int main(void) {
    unknown file = File.open("output.txt");
    file.write_line("Hello, World!");
    file.close();
    return 0;
}
