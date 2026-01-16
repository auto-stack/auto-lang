#pragma once

int str_len(char* self);
char* str_append(char* self, char* other);
char* str_upper(char* self);
char* str_lower(char* self);
char* str_sub(char* self, int start, int end);
bool str_contains(char* self, char* pattern);
bool str_starts_with(char* self, char* prefix);
bool str_ends_with(char* self, char* suffix);
int str_find(char* self, char* pattern);
char* str_trim(char* self);
char* str_trim_left(char* self);
char* str_trim_right(char* self);
char* str_replace(char* self, char* from, char* to);
int str_compare(char* self, char* other);
bool str_eq_ignore_case(char* self, char* other);
char* str_repeat(char* self, int n);
char* str_char_at(char* self, int index);
char* str_to_cstr(char* self);
int str_char_count(char* self);
char** str_split(int* out_size, char* self, char* delimiter);
char** str_lines(int* out_size, char* self);
char** str_words(int* out_size, char* self);
