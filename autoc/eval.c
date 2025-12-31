/**
 * Evaluator Implementation
 * Evaluates AST nodes into values
 */

#include "autoc.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// ============================================================================
// Evaluator Creation
// ============================================================================

Evaler* evaler_new(Universe* universe) {
    Evaler* evaler = (Evaler*)malloc(sizeof(Evaler));
    evaler->universe = universe;
    evaler->mode = EVAL_MODE_SCRIPT;
    evaler->skip_check = false;
    return evaler;
}

void evaler_free(Evaler* evaler) {
    if (evaler) {
        free(evaler);
    }
}

// ============================================================================
// Expression Evaluation
// ============================================================================

Value* evaler_eval_expr(Evaler* evaler, Expr* expr) {
    if (!expr) return value_nil();

    switch (expr->kind) {
        case EXPR_BYTE:
            return value_byte(expr->u.byte_val);

        case EXPR_INT:
            return value_int(expr->u.int_val);

        case EXPR_UINT:
            return value_uint(expr->u.uint_val);

        case EXPR_I8:
            return value_int((int32_t)expr->u.i8_val);

        case EXPR_U8:
            return value_uint((uint32_t)expr->u.u8_val);

        case EXPR_I64:
            return value_int((int32_t)expr->u.i64_val);

        case EXPR_FLOAT:
        case EXPR_DOUBLE:
            return value_float(expr->u.float_val);

        case EXPR_BOOL:
            return value_bool(expr->u.bool_val);

        case EXPR_CHAR:
            return value_char(expr->u.char_val);

        case EXPR_STR:
        case EXPR_CSTR:
            return value_str(expr->u.str_val.data);

        case EXPR_NIL:
            return value_nil();

        case EXPR_NULL:
            return value_error("null");

        case EXPR_IDENT: {
            Value* value = universe_lookup(evaler->universe, expr->u.ident_val.data);
            if (value) {
                return value_clone(value);
            }
            return value_nil();
        }

        case EXPR_UNARY: {
            Value* operand = evaler_eval_expr(evaler, expr->u.unary.expr);
            Value* result = NULL;
            // Simple unary operators - in a full impl, track operator type
            result = value_neg(operand);
            value_free(operand);
            return result;
        }

        case EXPR_BINA: {
            Value* left = evaler_eval_expr(evaler, expr->u.bina.left);
            Value* right = evaler_eval_expr(evaler, expr->u.bina.right);

            Value* result = NULL;
            switch ((TokenKind)expr->u.bina.op) {
                case TOKEN_ADD:
                    result = value_add(left, right);
                    break;
                case TOKEN_SUB:
                    result = value_sub(left, right);
                    break;
                case TOKEN_STAR:
                    result = value_mul(left, right);
                    break;
                case TOKEN_DIV:
                    result = value_div(left, right);
                    break;
                case TOKEN_EQ:
                    result = value_eq(left, right);
                    break;
                case TOKEN_NEQ:
                    result = value_neq(left, right);
                    break;
                case TOKEN_LT:
                    result = value_lt(left, right);
                    break;
                case TOKEN_GT:
                    result = value_gt(left, right);
                    break;
                case TOKEN_LE:
                    result = value_le(left, right);
                    break;
                case TOKEN_GE:
                    result = value_ge(left, right);
                    break;
                case TOKEN_RANGE:
                    if (left->kind == VAL_INT && right->kind == VAL_INT) {
                        result = value_range(left->u.int_val, right->u.int_val, false);
                    } else {
                        result = value_error("type error in range");
                    }
                    break;
                case TOKEN_RANGEEQ:
                    if (left->kind == VAL_INT && right->kind == VAL_INT) {
                        result = value_range(left->u.int_val, right->u.int_val, true);
                    } else {
                        result = value_error("type error in range");
                    }
                    break;
                case TOKEN_ASN: {
                    // Assignment
                    if (expr->u.bina.left->kind == EXPR_IDENT) {
                        const char* name = expr->u.bina.left->u.ident_val.data;
                        // Set the variable (creates or updates)
                        universe_set(evaler->universe, name, value_clone(right));
                        result = value_clone(right);
                    } else {
                        result = value_error("invalid assignment target");
                    }
                    break;
                }
                default:
                    result = value_nil();
                    break;
            }

            value_free(left);
            value_free(right);
            return result;
        }

        case EXPR_ARRAY: {
            ValueArray arr;
            arr.values = NULL;
            arr.count = 0;
            arr.capacity = 0;

            for (size_t i = 0; i < expr->u.array.count; i++) {
                if (arr.count >= arr.capacity) {
                    arr.capacity = arr.capacity == 0 ? 8 : arr.capacity * 2;
                    arr.values = (Value**)realloc(arr.values, arr.capacity * sizeof(Value*));
                }
                arr.values[arr.count++] = evaler_eval_expr(evaler, expr->u.array.elems[i]);
            }

            return value_array(arr);
        }

        case EXPR_OBJECT: {
            ValueObject obj;
            obj.pairs = NULL;
            obj.count = 0;
            obj.capacity = 0;

            for (size_t i = 0; i < expr->u.object.count; i++) {
                if (obj.count >= obj.capacity) {
                    obj.capacity = obj.capacity == 0 ? 8 : obj.capacity * 2;
                    obj.pairs = (KeyValue*)realloc(obj.pairs, obj.capacity * sizeof(KeyValue));
                }
                KeyValue kv;
                kv.key = expr->u.object.pairs[i].key;
                kv.value = evaler_eval_expr(evaler, expr->u.object.pairs[i].value);
                obj.pairs[obj.count++] = kv;
            }

            return value_object(obj);
        }

        case EXPR_CALL: {
            fprintf(stderr, "[DEBUG] EXPR_CALL: Evaluating callee\n");
            // Evaluate callee
            Value* callee = evaler_eval_expr(evaler, expr->u.call.callee);
            fprintf(stderr, "[DEBUG] EXPR_CALL: callee=%p, kind=%d\n", (void*)callee, callee ? callee->kind : -1);

            // Evaluate arguments
            Value** args = NULL;
            size_t arg_count = 0;
            for (size_t i = 0; i < expr->u.call.count; i++) {
                if (arg_count >= expr->u.call.capacity) {
                    args = (Value**)realloc(args, (arg_count + 1) * sizeof(Value*));
                }
                args[arg_count++] = evaler_eval_expr(evaler, expr->u.call.args[i]);
            }
            fprintf(stderr, "[DEBUG] EXPR_CALL: arg_count=%zu\n", arg_count);

            Value* result = value_nil();

            // Handle built-in print function
            if (callee && callee->kind == VAL_STR && strcmp(callee->u.str_val.data, "print") == 0) {
                fprintf(stderr, "[DEBUG] EXPR_CALL: Calling print\n");
                for (size_t i = 0; i < arg_count; i++) {
                    const char* repr = value_repr(args[i]);
                    printf("%s", repr);
                    if (i < arg_count - 1) printf(" ");
                }
                printf("\n");
                result = value_void();
            }

            // Cleanup
            value_free(callee);
            for (size_t i = 0; i < arg_count; i++) {
                value_free(args[i]);
            }
            free(args);

            return result;
        }

        case EXPR_INDEX: {
            Value* array = evaler_eval_expr(evaler, expr->u.index.array);
            Value* index = evaler_eval_expr(evaler, expr->u.index.index);

            Value* result = value_nil();

            if (array && array->kind == VAL_ARRAY && index && index->kind == VAL_INT) {
                int32_t idx = index->u.int_val;
                if (idx >= 0 && idx < (int32_t)array->u.array_val.count) {
                    result = value_clone(array->u.array_val.values[idx]);
                } else {
                    result = value_error("index out of bounds");
                }
            }

            value_free(array);
            value_free(index);
            return result;
        }

        case EXPR_BLOCK: {
            Value* last_value = value_nil();
            for (size_t i = 0; i < expr->u.block.count; i++) {
                value_free(last_value);
                last_value = evaler_eval_stmt(evaler, expr->u.block.stmts[i]);
            }
            return last_value;
        }

        case EXPR_IF: {
            Value* cond = evaler_eval_expr(evaler, expr->u.if_expr.cond);
            bool is_true = value_is_true(cond);
            value_free(cond);

            if (is_true) {
                return evaler_eval_expr(evaler, expr->u.if_expr.then_body);
            } else if (expr->u.if_expr.else_body) {
                return evaler_eval_expr(evaler, expr->u.if_expr.else_body);
            }
            return value_void();
        }

        default:
            return value_nil();
    }
}

// ============================================================================
// Statement Evaluation
// ============================================================================

Value* evaler_eval_stmt(Evaler* evaler, Stmt* stmt) {
    if (!stmt) return value_void();

    switch (stmt->kind) {
        case STMT_EXPR:
            return evaler_eval_expr(evaler, stmt->u.expr);

        case STMT_STORE: {
            Value* init = evaler_eval_expr(evaler, stmt->u.store.expr);
            universe_set(evaler->universe, stmt->u.store.name.data, init);
            // Return the assigned value (cloned since universe_set took ownership)
            return value_clone(init);
        }

        case STMT_BLOCK: {
            // Enter new scope
            universe_enter_scope(evaler->universe, SCOPE_BLOCK);

            Value* result = value_void();
            for (size_t i = 0; i < stmt->u.block.count; i++) {
                value_free(result);
                result = evaler_eval_stmt(evaler, stmt->u.block.stmts[i]);
            }

            // Exit scope
            universe_exit_scope(evaler->universe);

            return result;
        }

        case STMT_IF: {
            Value* cond = evaler_eval_expr(evaler, stmt->u.if_stmt.cond);
            bool is_true = value_is_true(cond);
            value_free(cond);

            if (is_true) {
                return evaler_eval_stmt(evaler, stmt->u.if_stmt.then_body);
            } else if (stmt->u.if_stmt.else_body) {
                return evaler_eval_stmt(evaler, stmt->u.if_stmt.else_body);
            }
            return value_void();
        }

        case STMT_FOR: {
            Value* iter_value = evaler_eval_expr(evaler, stmt->u.for_stmt.iter);

            // Enter loop scope
            universe_enter_scope(evaler->universe, SCOPE_BLOCK);

            Value* result = value_void();

            if (iter_value && iter_value->kind == VAL_RANGE) {
                int32_t start = iter_value->u.range_val.start;
                int32_t end = iter_value->u.range_val.end;
                bool eq = iter_value->u.range_val.eq;

                for (int32_t i = start; eq ? i <= end : i < end; i++) {
                    Value* index_val = value_int(i);
                    universe_set(evaler->universe, stmt->u.for_stmt.var_name.data, index_val);

                    value_free(result);
                    result = evaler_eval_stmt(evaler, stmt->u.for_stmt.body);
                }
            } else if (iter_value && iter_value->kind == VAL_ARRAY) {
                for (size_t i = 0; i < iter_value->u.array_val.count; i++) {
                    Value* elem = iter_value->u.array_val.values[i];
                    universe_set(evaler->universe, stmt->u.for_stmt.var_name.data, value_clone(elem));

                    value_free(result);
                    result = evaler_eval_stmt(evaler, stmt->u.for_stmt.body);
                }
            }

            // Exit scope
            universe_exit_scope(evaler->universe);

            value_free(iter_value);
            return result;
        }

        default:
            return value_void();
    }
}

// ============================================================================
// Code Evaluation
// ============================================================================

Value* evaler_eval(Evaler* evaler, Code* code) {
    Value* result = value_void();

    for (size_t i = 0; i < code->count; i++) {
        value_free(result);
        result = evaler_eval_stmt(evaler, code->stmts[i]);
    }

    return result;
}
