#include "std_file_flush.h"

int main(void) {
    unknown file = File.open("output.txt");
    file.write_line("Hello");
    file.flush();
    file.close();
    return 0;
}
