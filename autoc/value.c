/**
 * Value Implementation
 */

#include "autoc.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>

// ============================================================================
// Value Constructors
// ============================================================================

Value* value_nil(void) {
    Value* v = (Value*)malloc(sizeof(Value));
    v->kind = VAL_NIL;
    return v;
}

Value* value_void(void) {
    Value* v = (Value*)malloc(sizeof(Value));
    v->kind = VAL_VOID;
    return v;
}

Value* value_bool(bool b) {
    Value* v = (Value*)malloc(sizeof(Value));
    v->kind = VAL_BOOL;
    v->u.bool_val = b;
    return v;
}

Value* value_int(int32_t i) {
    Value* v = (Value*)malloc(sizeof(Value));
    v->kind = VAL_INT;
    v->u.int_val = i;
    return v;
}

Value* value_uint(uint32_t u) {
    Value* v = (Value*)malloc(sizeof(Value));
    v->kind = VAL_UINT;
    v->u.uint_val = u;
    return v;
}

Value* value_float(double f) {
    Value* v = (Value*)malloc(sizeof(Value));
    v->kind = VAL_DOUBLE;
    v->u.float_val = f;
    return v;
}

Value* value_byte(uint8_t b) {
    Value* v = (Value*)malloc(sizeof(Value));
    v->kind = VAL_BYTE;
    v->u.byte_val = b;
    return v;
}

Value* value_char(char c) {
    Value* v = (Value*)malloc(sizeof(Value));
    v->kind = VAL_CHAR;
    v->u.char_val = c;
    return v;
}

Value* value_str(const char* s) {
    Value* v = (Value*)malloc(sizeof(Value));
    v->kind = VAL_STR;
    v->u.str_val = astr_new(s);
    return v;
}

Value* value_array(ValueArray arr) {
    Value* v = (Value*)malloc(sizeof(Value));
    v->kind = VAL_ARRAY;
    v->u.array_val = arr;
    return v;
}

Value* value_object(ValueObject obj) {
    Value* v = (Value*)malloc(sizeof(Value));
    v->kind = VAL_OBJECT;
    v->u.object_val = obj;
    return v;
}

Value* value_range(int32_t start, int32_t end, bool eq) {
    Value* v = (Value*)malloc(sizeof(Value));
    v->kind = VAL_RANGE;
    v->u.range_val.start = start;
    v->u.range_val.end = end;
    v->u.range_val.eq = eq;
    return v;
}

Value* value_error(const char* msg) {
    Value* v = (Value*)malloc(sizeof(Value));
    v->kind = VAL_ERROR;
    v->u.error_val = astr_new(msg);
    return v;
}

// ============================================================================
// Value Destructor
// ============================================================================

void value_free(Value* value) {
    if (!value) return;
    switch (value->kind) {
        case VAL_STR:
            astr_free(&value->u.str_val);
            break;
        case VAL_ARRAY:
            if (value->u.array_val.values) {
                for (size_t i = 0; i < value->u.array_val.count; i++) {
                    value_free(value->u.array_val.values[i]);
                }
                free(value->u.array_val.values);
            }
            break;
        case VAL_OBJECT:
            if (value->u.object_val.pairs) {
                for (size_t i = 0; i < value->u.object_val.count; i++) {
                    astr_free(&value->u.object_val.pairs[i].key);
                    value_free(value->u.object_val.pairs[i].value);
                }
                free(value->u.object_val.pairs);
            }
            break;
        case VAL_ERROR:
            astr_free(&value->u.error_val);
            break;
        default:
            break;
    }
    free(value);
}

Value* value_clone(Value* value) {
    if (!value) return NULL;
    switch (value->kind) {
        case VAL_NIL: return value_nil();
        case VAL_VOID: return value_void();
        case VAL_BOOL: return value_bool(value->u.bool_val);
        case VAL_BYTE: return value_byte(value->u.byte_val);
        case VAL_INT: return value_int(value->u.int_val);
        case VAL_UINT: return value_uint(value->u.uint_val);
        case VAL_DOUBLE: return value_float(value->u.float_val);
        case VAL_CHAR: return value_char(value->u.char_val);
        case VAL_STR:
            return value_str(value->u.str_val.data);
        case VAL_ARRAY: {
            // Clone array by cloning each element
            ValueArray arr;
            arr.values = NULL;
            arr.count = 0;
            arr.capacity = 0;
            for (size_t i = 0; i < value->u.array_val.count; i++) {
                if (arr.count >= arr.capacity) {
                    arr.capacity = arr.capacity == 0 ? 8 : arr.capacity * 2;
                    arr.values = (Value**)realloc(arr.values, arr.capacity * sizeof(Value*));
                }
                arr.values[arr.count++] = value_clone(value->u.array_val.values[i]);
            }
            return value_array(arr);
        }
        case VAL_RANGE:
            return value_range(value->u.range_val.start, value->u.range_val.end, value->u.range_val.eq);
        default:
            return value_nil();
    }
}

// ============================================================================
// Value Predicates
// ============================================================================

bool value_is_true(Value* value) {
    if (!value) return false;
    switch (value->kind) {
        case VAL_BOOL: return value->u.bool_val;
        case VAL_NIL: return false;
        case VAL_INT: return value->u.int_val != 0;
        case VAL_UINT: return value->u.uint_val != 0;
        case VAL_DOUBLE: return value->u.float_val != 0.0;
        case VAL_STR: return value->u.str_val.len > 0;
        default: return true;
    }
}

bool value_is_nil(Value* value) {
    return value && value->kind == VAL_NIL;
}

bool value_is_void(Value* value) {
    return value && value->kind == VAL_VOID;
}

bool value_is_error(Value* value) {
    return value && value->kind == VAL_ERROR;
}

// ============================================================================
// Value Representation
// ============================================================================

const char* value_repr(Value* value) {
    static char buffer[4096];
    if (!value) return "(null)";
    switch (value->kind) {
        case VAL_NIL: return "nil";
        case VAL_VOID: return "void";
        case VAL_BOOL:
            return value->u.bool_val ? "true" : "false";
        case VAL_BYTE:
            snprintf(buffer, sizeof(buffer), "0x%02X", value->u.byte_val);
            return buffer;
        case VAL_INT:
            snprintf(buffer, sizeof(buffer), "%d", value->u.int_val);
            return buffer;
        case VAL_UINT:
            snprintf(buffer, sizeof(buffer), "%uu", value->u.uint_val);
            return buffer;
        case VAL_DOUBLE:
            snprintf(buffer, sizeof(buffer), "%g", value->u.float_val);
            return buffer;
        case VAL_CHAR:
            snprintf(buffer, sizeof(buffer), "'%c'", value->u.char_val);
            return buffer;
        case VAL_STR:
            return value->u.str_val.data;
        case VAL_ARRAY: {
            // Build array representation like [1, 2, 3]
            int offset = snprintf(buffer, sizeof(buffer), "[");
            for (size_t i = 0; i < value->u.array_val.count; i++) {
                if (i > 0) {
                    offset += snprintf(buffer + offset, sizeof(buffer) - offset, ", ");
                }
                const char* elem_repr = value_repr(value->u.array_val.values[i]);
                offset += snprintf(buffer + offset, sizeof(buffer) - offset, "%s", elem_repr);
            }
            snprintf(buffer + offset, sizeof(buffer) - offset, "]");
            return buffer;
        }
        case VAL_RANGE:
            if (value->u.range_val.eq) {
                snprintf(buffer, sizeof(buffer), "%d..=%d", value->u.range_val.start, value->u.range_val.end);
            } else {
                snprintf(buffer, sizeof(buffer), "%d..%d", value->u.range_val.start, value->u.range_val.end);
            }
            return buffer;
        case VAL_ERROR:
            return value->u.error_val.data;
        default:
            return "(unknown)";
    }
}

// ============================================================================
// Arithmetic Operations
// ============================================================================

Value* value_add(Value* a, Value* b) {
    // Handle int + int
    if (a->kind == VAL_INT && b->kind == VAL_INT) {
        return value_int(a->u.int_val + b->u.int_val);
    }
    // Handle uint + uint
    if (a->kind == VAL_UINT && b->kind == VAL_UINT) {
        return value_uint(a->u.uint_val + b->u.uint_val);
    }
    // Handle double + double
    if (a->kind == VAL_DOUBLE && b->kind == VAL_DOUBLE) {
        return value_float(a->u.float_val + b->u.float_val);
    }
    // Handle int + double
    if (a->kind == VAL_INT && b->kind == VAL_DOUBLE) {
        return value_float((double)a->u.int_val + b->u.float_val);
    }
    // Handle double + int
    if (a->kind == VAL_DOUBLE && b->kind == VAL_INT) {
        return value_float(a->u.float_val + (double)b->u.int_val);
    }
    // Handle string concatenation
    if (a->kind == VAL_STR && b->kind == VAL_STR) {
        AutoStr result = astr_clone(&a->u.str_val);
        astr_append(&result, b->u.str_val.data);
        Value* v = value_str(result.data);
        astr_free(&result);
        return v;
    }
    return value_error("type error in +");
}

Value* value_sub(Value* a, Value* b) {
    if (a->kind == VAL_INT && b->kind == VAL_INT) {
        return value_int(a->u.int_val - b->u.int_val);
    }
    if (a->kind == VAL_UINT && b->kind == VAL_UINT) {
        return value_uint(a->u.uint_val - b->u.uint_val);
    }
    if (a->kind == VAL_DOUBLE && b->kind == VAL_DOUBLE) {
        return value_float(a->u.float_val - b->u.float_val);
    }
    // Handle int - double (type promotion)
    if (a->kind == VAL_INT && b->kind == VAL_DOUBLE) {
        return value_float((double)a->u.int_val - b->u.float_val);
    }
    if (a->kind == VAL_DOUBLE && b->kind == VAL_INT) {
        return value_float(a->u.float_val - (double)b->u.int_val);
    }
    return value_error("type error in -");
}

Value* value_mul(Value* a, Value* b) {
    if (a->kind == VAL_INT && b->kind == VAL_INT) {
        return value_int(a->u.int_val * b->u.int_val);
    }
    if (a->kind == VAL_UINT && b->kind == VAL_UINT) {
        return value_uint(a->u.uint_val * b->u.uint_val);
    }
    if (a->kind == VAL_DOUBLE && b->kind == VAL_DOUBLE) {
        return value_float(a->u.float_val * b->u.float_val);
    }
    // Handle int * double (type promotion)
    if (a->kind == VAL_INT && b->kind == VAL_DOUBLE) {
        return value_float((double)a->u.int_val * b->u.float_val);
    }
    if (a->kind == VAL_DOUBLE && b->kind == VAL_INT) {
        return value_float(a->u.float_val * (double)b->u.int_val);
    }
    return value_error("type error in *");
}

Value* value_div(Value* a, Value* b) {
    if (a->kind == VAL_INT && b->kind == VAL_INT) {
        if (b->u.int_val == 0) return value_error("division by zero");
        return value_int(a->u.int_val / b->u.int_val);
    }
    if (a->kind == VAL_UINT && b->kind == VAL_UINT) {
        if (b->u.uint_val == 0) return value_error("division by zero");
        return value_uint(a->u.uint_val / b->u.uint_val);
    }
    if (a->kind == VAL_DOUBLE && b->kind == VAL_DOUBLE) {
        if (b->u.float_val == 0.0) return value_error("division by zero");
        return value_float(a->u.float_val / b->u.float_val);
    }
    // Handle int / double (type promotion)
    if (a->kind == VAL_INT && b->kind == VAL_DOUBLE) {
        if (b->u.float_val == 0.0) return value_error("division by zero");
        return value_float((double)a->u.int_val / b->u.float_val);
    }
    if (a->kind == VAL_DOUBLE && b->kind == VAL_INT) {
        if (b->u.int_val == 0) return value_error("division by zero");
        return value_float(a->u.float_val / (double)b->u.int_val);
    }
    return value_error("type error in /");
}

Value* value_neg(Value* a) {
    if (a->kind == VAL_INT) {
        return value_int(-a->u.int_val);
    }
    if (a->kind == VAL_DOUBLE) {
        return value_float(-a->u.float_val);
    }
    return value_error("type error in unary -");
}

Value* value_not(Value* a) {
    return value_bool(!value_is_true(a));
}

// ============================================================================
// Comparison Operations
// ============================================================================

Value* value_eq(Value* a, Value* b) {
    if (a->kind != b->kind) {
        // Allow cross-type comparison between numeric types
        if ((a->kind == VAL_INT && b->kind == VAL_DOUBLE) ||
            (a->kind == VAL_DOUBLE && b->kind == VAL_INT)) {
            double av = a->kind == VAL_INT ? (double)a->u.int_val : a->u.float_val;
            double bv = b->kind == VAL_INT ? (double)b->u.int_val : b->u.float_val;
            return value_bool(av == bv);
        }
        return value_bool(false);
    }
    switch (a->kind) {
        case VAL_BOOL: return value_bool(a->u.bool_val == b->u.bool_val);
        case VAL_INT: return value_bool(a->u.int_val == b->u.int_val);
        case VAL_UINT: return value_bool(a->u.uint_val == b->u.uint_val);
        case VAL_DOUBLE: return value_bool(a->u.float_val == b->u.float_val);
        case VAL_STR: return value_bool(astr_eq(&a->u.str_val, &b->u.str_val));
        case VAL_NIL: return value_bool(true);
        default: return value_bool(false);
    }
}

Value* value_neq(Value* a, Value* b) {
    Value* eq = value_eq(a, b);
    bool result = !value_is_true(eq);
    value_free(eq);
    return value_bool(result);
}

Value* value_lt(Value* a, Value* b) {
    if (a->kind == VAL_INT && b->kind == VAL_INT) {
        return value_bool(a->u.int_val < b->u.int_val);
    }
    if (a->kind == VAL_UINT && b->kind == VAL_UINT) {
        return value_bool(a->u.uint_val < b->u.uint_val);
    }
    if (a->kind == VAL_DOUBLE && b->kind == VAL_DOUBLE) {
        return value_bool(a->u.float_val < b->u.float_val);
    }
    return value_error("type error in <");
}

Value* value_gt(Value* a, Value* b) {
    if (a->kind == VAL_INT && b->kind == VAL_INT) {
        return value_bool(a->u.int_val > b->u.int_val);
    }
    if (a->kind == VAL_UINT && b->kind == VAL_UINT) {
        return value_bool(a->u.uint_val > b->u.uint_val);
    }
    if (a->kind == VAL_DOUBLE && b->kind == VAL_DOUBLE) {
        return value_bool(a->u.float_val > b->u.float_val);
    }
    return value_error("type error in >");
}

Value* value_le(Value* a, Value* b) {
    if (a->kind == VAL_INT && b->kind == VAL_INT) {
        return value_bool(a->u.int_val <= b->u.int_val);
    }
    if (a->kind == VAL_UINT && b->kind == VAL_UINT) {
        return value_bool(a->u.uint_val <= b->u.uint_val);
    }
    if (a->kind == VAL_DOUBLE && b->kind == VAL_DOUBLE) {
        return value_bool(a->u.float_val <= b->u.float_val);
    }
    return value_error("type error in <=");
}

Value* value_ge(Value* a, Value* b) {
    if (a->kind == VAL_INT && b->kind == VAL_INT) {
        return value_bool(a->u.int_val >= b->u.int_val);
    }
    if (a->kind == VAL_UINT && b->kind == VAL_UINT) {
        return value_bool(a->u.uint_val >= b->u.uint_val);
    }
    if (a->kind == VAL_DOUBLE && b->kind == VAL_DOUBLE) {
        return value_bool(a->u.float_val >= b->u.float_val);
    }
    return value_error("type error in >=");
}
