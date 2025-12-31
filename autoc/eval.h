/**
 * Evaluator Interface
 * Expression and statement evaluation
 */

#ifndef EVAL_H
#define EVAL_H

#include "ast.h"
#include "universe.h"

typedef enum {
    EVAL_MODE_SCRIPT,
    EVAL_MODE_CONFIG,
    EVAL_MODE_TEMPLATE,
} EvalMode;

typedef struct {
    Universe* universe;
    EvalMode mode;
    bool skip_check;
} Evaler;

Evaler* evaler_new(Universe* universe);
void evaler_free(Evaler* evaler);
Value* evaler_eval(Evaler* evaler, Code* code);
Value* evaler_eval_stmt(Evaler* evaler, Stmt* stmt);
Value* evaler_eval_expr(Evaler* evaler, Expr* expr);

#endif // EVAL_H
