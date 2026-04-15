#include "bang_operator.h"

int main(void) {
    unknown list = List.new();
    list.push(1);
    list.push(2);
    list.push(3);

    unknown collected = list.iter().collect();

    say(collected.len());
    return 0;
}
