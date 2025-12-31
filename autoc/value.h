/**
 * Value Runtime
 * Runtime value types and operations
 */

#ifndef VALUE_H
#define VALUE_H

#include "common.h"

typedef enum {
    VAL_NIL,
    VAL_VOID,
    VAL_BOOL,
    VAL_BYTE,
    VAL_INT,
    VAL_UINT,
    VAL_FLOAT,
    VAL_DOUBLE,
    VAL_CHAR,
    VAL_STR,
    VAL_ARRAY,
    VAL_OBJECT,
    VAL_RANGE,
    VAL_ERROR,
} ValueKind;

typedef struct Value Value;

typedef struct {
    Value** values;
    size_t count;
    size_t capacity;
} ValueArray;

typedef struct {
    AutoStr key;
    Value* value;
} KeyValue;

typedef struct {
    KeyValue* pairs;
    size_t count;
    size_t capacity;
} ValueObject;

struct Value {
    ValueKind kind;
    union {
        bool bool_val;
        uint8_t byte_val;
        int32_t int_val;
        uint32_t uint_val;
        double float_val;
        char char_val;
        AutoStr str_val;
        ValueArray array_val;
        ValueObject object_val;
        struct {
            int32_t start;
            int32_t end;
            bool eq;
        } range_val;
        AutoStr error_val;
    } u;
};

// Value constructors
Value* value_nil(void);
Value* value_void(void);
Value* value_bool(bool b);
Value* value_int(int32_t i);
Value* value_uint(uint32_t u);
Value* value_float(double f);
Value* value_str(const char* s);
Value* value_byte(uint8_t b);
Value* value_char(char c);
Value* value_array(ValueArray arr);
Value* value_object(ValueObject obj);
Value* value_range(int32_t start, int32_t end, bool eq);
Value* value_error(const char* msg);

// Value operations
void value_free(Value* value);
Value* value_clone(Value* value);
const char* value_repr(Value* value);
bool value_is_true(Value* value);
bool value_is_nil(Value* value);
bool value_is_void(Value* value);
bool value_is_error(Value* value);

// Arithmetic operations
Value* value_add(Value* a, Value* b);
Value* value_sub(Value* a, Value* b);
Value* value_mul(Value* a, Value* b);
Value* value_div(Value* a, Value* b);
Value* value_neg(Value* a);
Value* value_not(Value* a);

// Comparison operations
Value* value_eq(Value* a, Value* b);
Value* value_neq(Value* a, Value* b);
Value* value_lt(Value* a, Value* b);
Value* value_gt(Value* a, Value* b);
Value* value_le(Value* a, Value* b);
Value* value_ge(Value* a, Value* b);

#endif // VALUE_H
