#include "field_access.at.h"



int main(void) {

    struct Point p1 = {.x = 10, .y = 20};
    printf("%s %d\n", "p1.x: ", p1.x);
    printf("%s %d\n", "p1.y: ", p1.y);


    struct Point p2 = {.x = 1, .y = 2};
    printf("%s %d\n", "p2.x: ", p2.x);
    printf("%s %d\n", "p2.y: ", p2.y);


    struct Point p3 = {.x = 0, .y = 0};
    p3.x = 100;
    p3.y = 200;
    printf("%s %d\n", "p3.x: ", p3.x);
    printf("%s %d\n", "p3.y: ", p3.y);


    struct Data d = {.name = "test", .value = 42, .active = true};
    printf("%s %d\n", "d.name: ", d.name);
    printf("%s %d\n", "d.value: ", d.value);
    printf("%s %d\n", "d.active: ", d.active);


    struct Point p4 = {.x = 5, .y = 10};
    printf("%s %d\n", "p4.x (first): ", p4.x);
    printf("%s %d\n", "p4.x (second): ", p4.x);
    printf("%s %d\n", "p4.y: ", p4.y);
    return 0;
}
