/**
 * AutoString Utilities
 * Dynamic string type with automatic memory management
 */

#ifndef ASTR_H
#define ASTR_H

#include "common.h"

AutoStr astr_new(const char* s);
AutoStr astr_from_len(const char* s, size_t len);
void astr_free(AutoStr* s);
AutoStr astr_clone(AutoStr* s);
bool astr_eq(AutoStr* a, AutoStr* b);
AutoStr astr_append(AutoStr* a, const char* s);
AutoStr astr_append_char(AutoStr* a, char c);

#endif // ASTR_H
