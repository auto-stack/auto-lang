#include "test_list_generic.at.h"

int main(void) {
    list_int list = List.new();
    list.push(42);
    list.push(100);
    unknown len = list.len();
    return list;
}
