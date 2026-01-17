#pragma once

#include <stddef.h>

typedef struct {
    char* buffer;
    size_t len;
    size_t capacity;
} StringBuilder;

// API - Creation
struct May* StringBuilder_new(size_t initial_capacity);
void StringBuilder_drop(struct StringBuilder* sb);

// API - Building
struct May* StringBuilder_append(struct StringBuilder* sb, const char* str);
struct May* StringBuilder_append_char(struct StringBuilder* sb, char c);
struct May* StringBuilder_append_int(struct StringBuilder* sb, int value);

// API - Finalization
char* StringBuilder_build(struct StringBuilder* sb);
void StringBuilder_clear(struct StringBuilder* sb);
size_t StringBuilder_len(struct StringBuilder* sb);
