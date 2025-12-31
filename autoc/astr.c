/**
 * AutoString Implementation
 */

#include "autoc.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

AutoStr astr_new(const char* s) {
    AutoStr str;
    if (s) {
        str.len = strlen(s);
        str.capacity = str.len + 1;
        str.data = (char*)malloc(str.capacity);
        memcpy(str.data, s, str.len + 1);
    } else {
        str.data = (char*)malloc(1);
        str.data[0] = '\0';
        str.len = 0;
        str.capacity = 1;
    }
    return str;
}

AutoStr astr_from_len(const char* s, size_t len) {
    AutoStr str;
    str.len = len;
    str.capacity = len + 1;
    str.data = (char*)malloc(str.capacity);
    if (s) {
        memcpy(str.data, s, len);
    }
    str.data[len] = '\0';
    return str;
}

void astr_free(AutoStr* s) {
    if (s && s->data) {
        free(s->data);
        s->data = NULL;
        s->len = 0;
        s->capacity = 0;
    }
}

AutoStr astr_clone(AutoStr* s) {
    AutoStr str;
    str.len = s->len;
    str.capacity = s->capacity;
    str.data = (char*)malloc(str.capacity);
    memcpy(str.data, s->data, s->len + 1);
    return str;
}

bool astr_eq(AutoStr* a, AutoStr* b) {
    if (a->len != b->len) return false;
    return memcmp(a->data, b->data, a->len) == 0;
}

AutoStr astr_append(AutoStr* a, const char* s) {
    if (!s || !s[0]) return *a;
    size_t s_len = strlen(s);
    size_t new_len = a->len + s_len;
    if (new_len + 1 > a->capacity) {
        a->capacity = new_len * 2 + 1;
        a->data = (char*)realloc(a->data, a->capacity);
    }
    memcpy(a->data + a->len, s, s_len + 1);
    a->len = new_len;
    return *a;
}

AutoStr astr_append_char(AutoStr* a, char c) {
    if (a->len + 1 >= a->capacity) {
        a->capacity = a->capacity * 2 + 1;
        a->data = (char*)realloc(a->data, a->capacity);
    }
    a->data[a->len++] = c;
    a->data[a->len] = '\0';
    return *a;
}
