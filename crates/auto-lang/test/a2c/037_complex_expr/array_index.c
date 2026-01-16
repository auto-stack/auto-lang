#include "array_index.at.h"

int main(void) {
    int arr[4] = {12, 13, 14, 15};
    int idx = 2;

    printf("%d\n", arr[2]);

    printf("%d\n", arr[idx]);

    printf("%d\n", arr[0 + 1]);
    return 0;
}
