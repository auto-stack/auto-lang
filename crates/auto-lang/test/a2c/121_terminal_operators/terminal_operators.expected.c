#include "terminal_operators.h"

int add(int a, int b) {
    return a + b;
}

void print_item(int x) {
    say(x);
}

int main(void) {

    list_void*_void* list = List.new();
    list.push(1);
    list.push(2);
    list.push(3);

    unknown sum = list.iter().reduce(0, add);
    say(sum);

    unknown count = list.iter().count();
    say(count);

    list.iter().for_each(print_item);
    return 0;
}
