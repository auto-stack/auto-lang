#include "dstr.at.h"

dstr dstr_New(struct dstr *self) {
    
    return {};
}
dstr dstr_FromByte(struct dstr *self, char) {
    unknown list = List.new();
    list.push(b);
    
    return {};
}
dstr dstr_FromBytes(struct dstr *self, char, char) {
    unknown list = List.new();
    list.push(b1);
    list.push(b2);
    
    return {};
}
int dstr_Len(struct dstr *self) {
    List data = self->data;
    data.len();
    return 0;
}
bool dstr_IsEmpty(struct dstr *self) {
    List data = self->data;
    data.is_empty();
    return false;
}
char dstr_Get(struct dstr *self, int) {
    List data = self->data;
    data.get(index);
    return 0;
}
void dstr_Push(struct dstr *self, char) {
    self->data.push(b);
}
char dstr_Pop(struct dstr *self) {
    self->data.pop();
    return 0;
}
int dstr_Set(struct dstr *self, int, char) {
    self->data.set(index, b);
    return 0;
}
void dstr_Insert(struct dstr *self, int, char) {
    self->data.insert(index, b);
}
char dstr_Remove(struct dstr *self, int) {
    self->data.remove(index);
    return 0;
}
void dstr_Clear(struct dstr *self) {
    self->data.clear();
}
void dstr_Reserve(struct dstr *self, int) {
    self->data.reserve(capacity);
}
