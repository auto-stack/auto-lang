#include "collect.h"

int double(int x) {
    return x * 2;
}

bool is_gt_3(int x) {
    return x > 3;
}

int main(void) {

    list_void*_void* list = List.new();
    list.push(1);
    list.push(2);
    list.push(3);
    list.push(4);
    list.push(5);

    unknown mapped = list.iter().map(double).collect();
    say(mapped.len());

    unknown filtered = list.iter().filter(is_gt_3).collect();
    say(filtered.len());

    unknown chained = list.iter().map(double).filter(is_gt_3).collect();
    say(chained.len());
    return 0;
}
