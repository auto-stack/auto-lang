#include "std_path.h"

int main(void) {

    char* result = join("/home/user", "file.txt");
    printf("%s\n", result);

    bool abs = is_absolute("/home/user");
    printf("%d\n", abs);

    bool rel = is_relative("user/file");
    printf("%d\n", rel);
    return 0;
}
