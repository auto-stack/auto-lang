#include "list_capacity.h"

int main(void) {

    unknown list = List.new();

    list.push(1);
    list.push(2);
    list.push(3);

    unknown cap = list.capacity();

    unknown len = list.len();

    printf("%s %d\n", "List capacity:", cap);
    printf("%s %d\n", "List length:", len);
    return 0;
}
