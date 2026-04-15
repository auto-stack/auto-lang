#include "std_readline_2.h"

int main(void) {
    unknown file = File.open("test_lines.txt");
    unknown line1 = file.read_line();
    unknown line2 = file.read_line();
    file.close();
    return 0;
}
