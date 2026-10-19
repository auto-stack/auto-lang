/*
 * interop_resource_b.c — PLAN-797 F-797-R4-05 permanent counterexample
 * fixture: a SECOND, independently compiled provider whose exported
 * symbols are renamed (exactly the review's construction). It owns a
 * separate arena/slot table and a distinct provider identity token, so
 * handles minted by provider A must be rejected here and vice versa.
 * No _fltused: both providers stay FP-literal-free; the linked C caller
 * owns that definition.
 */

#define p797_h_create      b_h_create
#define p797_h_set         b_h_set
#define p797_h_get         b_h_get
#define p797_h_destroy     b_h_destroy
#define p797_ledger        b_ledger
#define p797_buf_sum       b_buf_sum
#define p797_buf_copy      b_buf_copy
#define p797_fail_nth_alloc b_fail_nth_alloc
#define p797_two_step      b_two_step
#define p797_fold          b_fold
#define p797_stored_cb     b_stored_cb
#define p797_cbuf_sum_utf8 b_cbuf_sum_utf8
#define p797_crec_valid    b_crec_valid
#define p797_crec_mutate   b_crec_mutate
#define p797_crec_make     b_crec_make

#include "interop_resource.c"
