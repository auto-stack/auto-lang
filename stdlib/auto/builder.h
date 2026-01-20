#pragma once

struct StringBuilder {
    char* buffer;
    int len;
    int capacity;
};

StringBuilder StringBuilder_New(struct StringBuilder *self, int);
StringBuilder StringBuilder_NewWithDefault(struct StringBuilder *self);
void StringBuilder_Append(struct StringBuilder *self, str);
void StringBuilder_AppendChar(struct StringBuilder *self, char);
void StringBuilder_AppendInt(struct StringBuilder *self, int);
str StringBuilder_Build(struct StringBuilder *self);
void StringBuilder_Clear(struct StringBuilder *self);
int StringBuilder_Len(struct StringBuilder *self);
void StringBuilder_Drop(struct StringBuilder *self);
