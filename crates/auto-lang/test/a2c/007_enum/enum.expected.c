#include <stdio.h>

enum Color {
    COLOR_RED = 1,
    COLOR_GREEN = 2,
    COLOR_BLUE = 3,
};

int main(void) {
    enum Color color = COLOR_BLUE;
    printf("%s %d\n", "The color is", color);
    return 0;
}
