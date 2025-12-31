/**
 * Common Types
 * Basic types used throughout the compiler
 */

#ifndef COMMON_H
#define COMMON_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

typedef struct {
    char* data;
    size_t len;
    size_t capacity;
} AutoStr;

typedef struct {
    size_t line;
    size_t at;
    size_t pos;
    size_t len;
} Pos;

#endif // COMMON_H
