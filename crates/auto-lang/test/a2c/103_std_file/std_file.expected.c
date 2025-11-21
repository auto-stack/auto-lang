#include <stdio.h>

int main(void) {
    FILE* fp = NULL;
    fp = fopen("Cargo.toml", "r");

    char buf[100] = {0};
    while (fgets(buf, 100, fp)) {
        printf("%s", buf);
    }

    fclose(fp);
    return 0;
}
