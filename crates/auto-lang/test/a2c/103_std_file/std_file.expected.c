#include "std_file.h"

void test_file_method(void) {
    struct File file = open("Cargo.toml");
    char* line = File_ReadText(&file);
    printf("%s\n", line);
    File_Close(&file);
}

void test_c_file_functions(void) {
    FILE* fp = NULL;
    fp = fopen("Cargo.toml", "r");

    char buf[100] = {0};
    while (fgets(buf, 100, fp)) {
        printf("%s", buf);
    }

    fclose(fp);
}

int main(void) {
    test_file_method();
    test_c_file_functions();
    return 0;
}
