warning: unused import: `std::collections::HashMap`
  --> crates\auto-lang\src\autovm_persistent.rs:18:5
   |
18 | use std::collections::HashMap;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `crate::vm::native::ShimFunc`
  --> crates\auto-lang\src\ffi.rs:16:5
   |
16 | use crate::vm::native::ShimFunc;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `StoreKind`, `Store`, and `Type`
 --> crates\auto-lang\src\typeck\param_check.rs:4:57
  |
4 | use crate::ast::{Fn, Stmt, Expr, Name, ParamMode, Body, Type, Store, StoreKind};
  |                                                         ^^^^  ^^^^^  ^^^^^^^^^

warning: unused imports: `BTreeMap` and `VecDeque`
  --> crates\auto-lang\src\universe.rs:15:24
   |
15 | use std::collections::{BTreeMap, HashMap as StdHashMap, VecDeque};
   |                        ^^^^^^^^                         ^^^^^^^^

warning: unused import: `try_downcast_checked_mut`
   --> crates\auto-lang\src\vm\engine.rs:842:50
    |
842 |                     use crate::vm::heap_object::{try_downcast_checked_mut, TypeTag};
    |                                                  ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `Member` and `TypeDecl`
 --> crates\auto-lang\src\vm\generic_registry.rs:4:36
  |
4 | use crate::ast::{Fn, GenericParam, Member, Name, Type, TypeDecl};
  |                                    ^^^^^^              ^^^^^^^^

warning: unused import: `AutoStr`
 --> crates\auto-lang\src\vm\generic_registry.rs:6:16
  |
6 | use auto_val::{AutoStr, Value};
  |                ^^^^^^^

warning: unused import: `RwLock`
  --> crates\auto-lang\src\vm\generic_registry.rs:10:22
   |
10 | use std::sync::{Arc, RwLock};
   |                      ^^^^^^

warning: unused import: `std::collections::HashMap`
 --> crates\auto-lang\src\vm\monomorphize.rs:7:5
  |
7 | use std::collections::HashMap;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::sync::atomic::Ordering`
   --> crates\auto-lang\src\vm\native.rs:679:9
    |
679 |     use std::sync::atomic::Ordering;
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::vm::heap_object::HeapObject`
   --> crates\auto-lang\src\vm\native.rs:832:9
    |
832 |     use crate::vm::heap_object::HeapObject;
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-lang\src\lib.rs:66:43
   |
66 | pub use crate::universe::{SymbolLocation, Universe};
   |                                           ^^^^^^^^
   |
   = note: `#[warn(deprecated)]` on by default

warning: use of deprecated enum `eval::EvalMode`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
  --> crates\auto-lang\src\lib.rs:68:19
   |
68 | use crate::{eval::EvalMode, trans::Sink, trans::Trans};
   |                   ^^^^^^^^

warning: use of deprecated struct `interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\lib.rs:252:35
    |
252 |     let mut interpreter = interp::Interpreter::new();
    |                                   ^^^^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\lib.rs:270:42
    |
270 | pub fn run_with_scope(code: &str, scope: Universe) -> AutoResult<String> {
    |                                          ^^^^^^^^

warning: use of deprecated struct `interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\lib.rs:271:35
    |
271 |     let mut interpreter = interp::Interpreter::with_scope(scope);
    |                                   ^^^^^^^^^^^

warning: use of deprecated struct `interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\lib.rs:315:35
    |
315 |     let mut interpreter = interp::Interpreter::new_with_session(session);
    |                                   ^^^^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\lib.rs:342:19
    |
342 |     scope: Shared<Universe>,
    |                   ^^^^^^^^

warning: use of deprecated struct `interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\lib.rs:350:35
    |
350 |     let mut interpreter = interp::Interpreter::new_with_session_and_scope(session, scope);
    |                                   ^^^^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\lib.rs:360:38
    |
360 |     let scope = Rc::new(RefCell::new(Universe::new()));
    |                                      ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\lib.rs:368:38
    |
368 |     let scope = Rc::new(RefCell::new(Universe::new()));
    |                                      ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\lib.rs:373:55
    |
373 | pub fn parse_with_scope(code: &str, scope: Rc<RefCell<Universe>>) -> AutoResult<ast::Code> {
    |                                                       ^^^^^^^^

warning: use of deprecated struct `interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\lib.rs:378:52
    |
378 | pub fn interpret(code: &str) -> AutoResult<interp::Interpreter> {
    |                                                    ^^^^^^^^^^^

warning: use of deprecated struct `interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\lib.rs:379:35
    |
379 |     let mut interpreter = interp::Interpreter::new();
    |                                   ^^^^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\lib.rs:384:48
    |
384 | pub fn interpret_with_scope(code: &str, scope: Universe) -> AutoResult<interp::Interpreter> {
    |                                                ^^^^^^^^

warning: use of deprecated struct `interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\lib.rs:384:80
    |
384 | pub fn interpret_with_scope(code: &str, scope: Universe) -> AutoResult<interp::Interpreter> {
    |                                                                                ^^^^^^^^^^^

warning: use of deprecated struct `interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\lib.rs:385:35
    |
385 |     let mut interpreter = interp::Interpreter::with_scope(scope);
    |                                   ^^^^^^^^^^^

warning: use of deprecated struct `interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\lib.rs:398:46
    |
398 | pub fn interpret_file(path: &str) -> interp::Interpreter {
    |                                              ^^^^^^^^^^^

warning: use of deprecated struct `interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\lib.rs:402:35
    |
402 |     let mut interpreter = interp::Interpreter::new();
    |                                   ^^^^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\lib.rs:408:45
    |
408 | pub fn eval_template(template: &str, scope: Universe) -> AutoResult<interp::Interpreter> {
    |                                             ^^^^^^^^

warning: use of deprecated struct `interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\lib.rs:408:77
    |
408 | pub fn eval_template(template: &str, scope: Universe) -> AutoResult<interp::Interpreter> {
    |                                                                             ^^^^^^^^^^^

warning: use of deprecated struct `interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\lib.rs:409:35
    |
409 |     let mut interpreter = interp::Interpreter::with_scope(scope).with_eval_mode(EvalMode::TEMPLATE);
    |                                   ^^^^^^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::TEMPLATE`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
   --> crates\auto-lang\src\lib.rs:409:91
    |
409 |     let mut interpreter = interp::Interpreter::with_scope(scope).with_eval_mode(EvalMode::TEMPLATE);
    |                                                                                           ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\lib.rs:418:16
    |
418 |     mut scope: Universe,
    |                ^^^^^^^^

warning: use of deprecated struct `interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\lib.rs:419:25
    |
419 | ) -> AutoResult<interp::Interpreter> {
    |                         ^^^^^^^^^^^

warning: use of deprecated struct `interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\lib.rs:424:35
    |
424 |     let mut interpreter = interp::Interpreter::with_scope(scope).with_eval_mode(EvalMode::CONFIG);
    |                                   ^^^^^^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::CONFIG`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
   --> crates\auto-lang\src\lib.rs:424:91
    |
424 |     let mut interpreter = interp::Interpreter::with_scope(scope).with_eval_mode(EvalMode::CONFIG);
    |                                                                                           ^^^^^^

warning: use of deprecated struct `interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\lib.rs:429:66
    |
429 | pub fn eval_config(code: &str, args: &Obj) -> AutoResult<interp::Interpreter> {
    |                                                                  ^^^^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\lib.rs:432:21
    |
432 |     let mut scope = Universe::new();
    |                     ^^^^^^^^

warning: use of deprecated struct `interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\lib.rs:435:35
    |
435 |     let mut interpreter = interp::Interpreter::with_scope(scope).with_eval_mode(EvalMode::CONFIG);
    |                                   ^^^^^^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::CONFIG`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
   --> crates\auto-lang\src\lib.rs:435:91
    |
435 |     let mut interpreter = interp::Interpreter::with_scope(scope).with_eval_mode(EvalMode::CONFIG);
    |                                                                                           ^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\lib.rs:464:58
    |
464 | pub fn eval_config_with_vm(code: &str, args: &Obj, univ: Universe) -> AutoResult<Value> {
    |                                                          ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\lib.rs:565:38
    |
565 |     let scope = Rc::new(RefCell::new(Universe::new()));
    |                                      ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\lib.rs:591:38
    |
591 |     let scope = Rc::new(RefCell::new(Universe::new()));
    |                                      ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\lib.rs:759:38
    |
759 |     let scope = Rc::new(RefCell::new(Universe::new()));
    |                                      ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\lib.rs:782:38
    |
782 |     let scope = Rc::new(RefCell::new(Universe::new()));
    |                                      ^^^^^^^^

warning: use of deprecated struct `interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\atom.rs:704:28
    |
704 |     interp: crate::interp::Interpreter,
    |                            ^^^^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\atom.rs:707:35
    |
707 |     univ: auto_val::Shared<crate::Universe>,
    |                                   ^^^^^^^^

warning: use of deprecated enum `eval::EvalMode`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
   --> crates\auto-lang\src\atom.rs:721:26
    |
721 |         use crate::eval::EvalMode;
    |                          ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\atom.rs:724:34
    |
724 |         let univ = shared(crate::Universe::new());
    |                                  ^^^^^^^^

warning: use of deprecated struct `interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\atom.rs:726:28
    |
726 |             crate::interp::Interpreter::with_univ(univ.clone()).with_eval_mode(EvalMode::CONFIG);
    |                            ^^^^^^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::CONFIG`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
   --> crates\auto-lang\src\atom.rs:726:90
    |
726 |             crate::interp::Interpreter::with_univ(univ.clone()).with_eval_mode(EvalMode::CONFIG);
    |                                                                                          ^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\compile.rs:130:59
    |
130 |         let scope = Rc::new(RefCell::new(crate::universe::Universe::new()));
    |                                                           ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
 --> crates\auto-lang\src\config.rs:5:12
  |
5 | use crate::Universe;
  |            ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-lang\src\config.rs:15:22
   |
15 |     pub univ: Shared<Universe>,
   |                      ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-lang\src\config.rs:20:27
   |
20 |         let univ = shared(Universe::new());
   |                           ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-lang\src\config.rs:42:70
   |
42 |         let result = eval_config_with_vm(code.as_str(), &Obj::new(), Universe::new())?;
   |                                                                      ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-lang\src\config.rs:72:48
   |
72 |         Self::from_file(path, &Obj::default(), Universe::default())
   |                                                ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-lang\src\config.rs:75:53
   |
75 |     pub fn from_file(path: &Path, args: &Obj, univ: Universe) -> AutoResult<Self> {
   |                                                     ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-lang\src\config.rs:87:20
   |
87 |         let univ = Universe::default();
   |                    ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-lang\src\config.rs:98:65
   |
98 |     pub fn from_code(code: impl Into<String>, args: &Obj, univ: Universe) -> AutoResult<Self> {
   |                                                                 ^^^^^^^^

warning: use of deprecated struct `interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
  --> crates\auto-lang\src\execution_engine.rs:65:27
   |
65 |     let mut interpreter = Interpreter::new();
   |                           ^^^^^^^^^^^

warning: use of deprecated struct `interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
  --> crates\auto-lang\src\execution_engine.rs:64:24
   |
64 |     use crate::interp::Interpreter;
   |                        ^^^^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
 --> crates\auto-lang\src\eval.rs:6:22
  |
6 | use crate::universe::Universe;
  |                      ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-lang\src\eval.rs:85:26
   |
85 |     universe: Rc<RefCell<Universe>>,
   |                          ^^^^^^^^

warning: use of deprecated enum `eval::EvalMode`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
  --> crates\auto-lang\src\eval.rs:93:11
   |
93 |     mode: EvalMode,
   |           ^^^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:107:6
    |
107 | impl Evaler {
    |      ^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\eval.rs:108:37
    |
108 |     pub fn new(universe: Rc<RefCell<Universe>>) -> Self {
    |                                     ^^^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:112:25
    |
112 |         let evaluator = Evaler {
    |                         ^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::SCRIPT`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
   --> crates\auto-lang\src\eval.rs:121:29
    |
121 |             mode: EvalMode::SCRIPT,
    |                             ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:142:37
    |
142 |         let eval_ptr = self as *mut Evaler;
    |                                     ^^^^^^

warning: use of deprecated enum `eval::EvalMode`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
   --> crates\auto-lang\src\eval.rs:148:38
    |
148 |     pub fn with_mode(mut self, mode: EvalMode) -> Self {
    |                                      ^^^^^^^^

warning: use of deprecated enum `eval::EvalMode`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
   --> crates\auto-lang\src\eval.rs:153:38
    |
153 |     pub fn set_mode(&mut self, mode: EvalMode) {
    |                                      ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\eval.rs:207:43
    |
207 |     pub fn universe(&self) -> &Rc<RefCell<Universe>> {
    |                                           ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\eval.rs:220:55
    |
220 |     pub fn universe_mut(&mut self) -> &mut Rc<RefCell<Universe>> {
    |                                                       ^^^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::SCRIPT`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
   --> crates\auto-lang\src\eval.rs:721:23
    |
721 |             EvalMode::SCRIPT => {
    |                       ^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::CONFIG`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
   --> crates\auto-lang\src\eval.rs:741:23
    |
741 |             EvalMode::CONFIG => {
    |                       ^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::TEMPLATE`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
   --> crates\auto-lang\src\eval.rs:986:23
    |
986 |             EvalMode::TEMPLATE => {
    |                       ^^^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::SCRIPT`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
    --> crates\auto-lang\src\eval.rs:1257:23
     |
1257 |             EvalMode::SCRIPT => Ok(res.last().unwrap_or(&Value::Nil).clone()),
     |                       ^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::CONFIG`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
    --> crates\auto-lang\src\eval.rs:1258:23
     |
1258 |             EvalMode::CONFIG => Ok(Value::Block(Array::from_vec(self.collect_config_body(res)))),
     |                       ^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::TEMPLATE`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
    --> crates\auto-lang\src\eval.rs:1259:23
     |
1259 |             EvalMode::TEMPLATE => Ok(Value::Str(
     |                       ^^^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::SCRIPT`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
    --> crates\auto-lang\src\eval.rs:1293:23
     |
1293 |             EvalMode::SCRIPT => res.last().unwrap_or(&Value::Nil).clone(),
     |                       ^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::CONFIG`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
    --> crates\auto-lang\src\eval.rs:1294:23
     |
1294 |             EvalMode::CONFIG => Value::Array(Array::from_vec(self.collect_config_body(res))),
     |                       ^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::TEMPLATE`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
    --> crates\auto-lang\src\eval.rs:1295:23
     |
1295 |             EvalMode::TEMPLATE => Value::Str(
     |                       ^^^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::SCRIPT`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
    --> crates\auto-lang\src\eval.rs:1485:27
     |
1485 |                 EvalMode::SCRIPT => Value::Void,
     |                           ^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::CONFIG`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
    --> crates\auto-lang\src\eval.rs:1486:27
     |
1486 |                 EvalMode::CONFIG => Value::Array(res),
     |                           ^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::TEMPLATE`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
    --> crates\auto-lang\src\eval.rs:1487:27
     |
1487 |                 EvalMode::TEMPLATE => Value::Str(
     |                           ^^^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::SCRIPT`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
    --> crates\auto-lang\src\eval.rs:1553:27
     |
1553 |                 EvalMode::SCRIPT => Value::Void,
     |                           ^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::CONFIG`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
    --> crates\auto-lang\src\eval.rs:1554:27
     |
1554 |                 EvalMode::CONFIG => Value::Array(res),
     |                           ^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::TEMPLATE`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
    --> crates\auto-lang\src\eval.rs:1555:27
     |
1555 |                 EvalMode::TEMPLATE => Value::Str(
     |                           ^^^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::SCRIPT`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
    --> crates\auto-lang\src\eval.rs:1715:43
     |
1715 | ...                   EvalMode::SCRIPT => Value::Void,
     |                                 ^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::CONFIG`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
    --> crates\auto-lang\src\eval.rs:1716:43
     |
1716 | ...                   EvalMode::CONFIG => Value::Array(res),
     |                                 ^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::TEMPLATE`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
    --> crates\auto-lang\src\eval.rs:1717:43
     |
1717 | ...                   EvalMode::TEMPLATE => Value::Str(
     |                                 ^^^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::SCRIPT`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
    --> crates\auto-lang\src\eval.rs:1747:27
     |
1747 |                 EvalMode::SCRIPT => Value::Void,
     |                           ^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::CONFIG`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
    --> crates\auto-lang\src\eval.rs:1748:27
     |
1748 |                 EvalMode::CONFIG => Value::Array(res),
     |                           ^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::TEMPLATE`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
    --> crates\auto-lang\src\eval.rs:1749:27
     |
1749 |                 EvalMode::TEMPLATE => Value::Str(
     |                           ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\indexer.rs:442:70
    |
442 |         let scope = Rc::new(std::cell::RefCell::new(crate::universe::Universe::new()));
    |                                                                      ^^^^^^^^

warning: use of deprecated enum `eval::EvalMode`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
 --> crates\auto-lang\src\interp.rs:2:19
  |
2 | use crate::eval::{EvalMode, Evaler};
  |                   ^^^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
 --> crates\auto-lang\src\interp.rs:2:29
  |
2 | use crate::eval::{EvalMode, Evaler};
  |                             ^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
 --> crates\auto-lang\src\interp.rs:5:22
  |
5 | use crate::universe::Universe;
  |                      ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-lang\src\interp.rs:14:23
   |
14 |     pub scope: Shared<Universe>,
   |                       ^^^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
  --> crates\auto-lang\src\interp.rs:57:17
   |
57 |     pub evaler: Evaler,
   |                 ^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-lang\src\interp.rs:61:23
   |
61 |     pub scope: Shared<Universe>,
   |                       ^^^^^^^^

warning: use of deprecated struct `interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
  --> crates\auto-lang\src\interp.rs:77:6
   |
77 | impl Interpreter {
   |      ^^^^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-lang\src\interp.rs:88:28
   |
88 |         let scope = shared(Universe::new());
   |                            ^^^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
  --> crates\auto-lang\src\interp.rs:93:21
   |
93 |             evaler: Evaler::new(scope.clone()),
   |                     ^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\interp.rs:181:41
    |
181 |     fn load_stdlib_types(scope: &Shared<Universe>) {
    |                                         ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\interp.rs:280:35
    |
280 |     pub fn with_univ(univ: Shared<Universe>) -> Self {
    |                                   ^^^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\interp.rs:292:21
    |
292 |             evaler: Evaler::new(univ.clone()),
    |                     ^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\interp.rs:318:30
    |
318 |     pub fn with_scope(scope: Universe) -> Self {
    |                              ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\interp.rs:339:28
    |
339 |         let scope = shared(Universe::new());
    |                            ^^^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\interp.rs:344:21
    |
344 |             evaler: Evaler::new(scope.clone()),
    |                     ^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\interp.rs:425:79
    |
425 |     pub fn new_with_session_and_scope(session: &CompileSession, scope: Shared<Universe>) -> Self {
    |                                                                               ^^^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\interp.rs:438:21
    |
438 |             evaler: Evaler::new(scope.clone()),
    |                     ^^^^^^

warning: use of deprecated enum `eval::EvalMode`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
   --> crates\auto-lang\src\interp.rs:473:43
    |
473 |     pub fn with_eval_mode(mut self, mode: EvalMode) -> Self {
    |                                           ^^^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::TEMPLATE`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
   --> crates\auto-lang\src\interp.rs:567:40
    |
567 |         self.evaler.set_mode(EvalMode::TEMPLATE);
    |                                        ^^^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\interp.rs:610:33
    |
610 |         let mut config_evaler = Evaler::new(self.scope.clone()).with_mode(EvalMode::CONFIG);
    |                                 ^^^^^^

warning: use of deprecated unit variant `eval::EvalMode::CONFIG`: Use AutoVM with Codegen/ConfigCodegen/TemplateCodegen instead (Plan 068 Phase 9 + Plan 075)
   --> crates\auto-lang\src\interp.rs:610:85
    |
610 |         let mut config_evaler = Evaler::new(self.scope.clone()).with_mode(EvalMode::CONFIG);
    |                                                                                     ^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
 --> crates\auto-lang\src\parser.rs:7:39
  |
7 | use crate::universe::{SymbolLocation, Universe};
  |                                       ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\parser.rs:156:23
    |
156 |     pub scope: Shared<Universe>,
    |                       ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\parser.rs:181:32
    |
181 |         Self::new(code, shared(Universe::new()))
    |                                ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\parser.rs:184:45
    |
184 |     pub fn new(code: &'a str, scope: Shared<Universe>) -> Self {
    |                                             ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\parser.rs:231:55
    |
231 |     pub fn new_with_note(code: &'a str, scope: Shared<Universe>, note: char) -> Self {
    |                                                       ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\parser.rs:266:23
    |
266 |         scope: Shared<Universe>,
    |                       ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
 --> crates\auto-lang\src\repl.rs:5:23
  |
5 | use crate::universe::{Universe, VmRefData};
  |                       ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-lang\src\repl.rs:26:23
   |
26 |     pub scope: Shared<Universe>,
   |                       ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-lang\src\repl.rs:97:45
   |
97 | fn format_value(value: &Value, uni: &Shared<Universe>) -> String {
   |                                             ^^^^^^^^

warning: use of deprecated struct `interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\repl.rs:174:54
    |
174 | fn try_command(line: &str, interpreter: &mut interp::Interpreter) -> CmdResult {
    |                                                      ^^^^^^^^^^^

warning: use of deprecated struct `interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
  --> crates\auto-lang\src\repl.rs:36:44
   |
36 |         let mut temp_interpreter = interp::Interpreter::new();
   |                                            ^^^^^^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
  --> crates\auto-lang\src\runtime.rs:16:18
   |
16 | use crate::eval::Evaler;
   |                  ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\runtime.rs:167:25
    |
167 |     evaluator_ptr: *mut Evaler,
    |                         ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\runtime.rs:351:53
    |
351 |     pub fn set_evaluator(&mut self, evaluator: &mut Evaler) {
    |                                                     ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\runtime.rs:358:64
    |
358 |     pub unsafe fn set_evaluator_raw(&mut self, evaluator: *mut Evaler) {
    |                                                                ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\runtime.rs:380:52
    |
380 |     pub unsafe fn get_evaluator_ptr(&self) -> *mut Evaler {
    |                                                    ^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
 --> crates\auto-lang\src\trans\c.rs:8:22
  |
8 | use crate::universe::Universe;
  |                      ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-lang\src\trans\c.rs:42:26
   |
42 |     scope: Option<Shared<Universe>>,      // Old (deprecated)
   |                          ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
    --> crates\auto-lang\src\trans\c.rs:4045:38
     |
4045 |     let scope = Rc::new(RefCell::new(Universe::new()));
     |                                      ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
    --> crates\auto-lang\src\trans\c.rs:4060:86
     |
4060 | pub fn transpile_c(name: impl Into<AutoStr>, code: &str) -> AutoResult<(Sink, Shared<Universe>)> {
     |                                                                                      ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
    --> crates\auto-lang\src\trans\c.rs:4062:38
     |
4062 |     let scope = Rc::new(RefCell::new(Universe::new()));
     |                                      ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-lang\src\trans\c.rs:68:32
   |
68 |             scope: Some(shared(Universe::default())),
   |                                ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-lang\src\trans\c.rs:96:47
   |
96 |     pub fn set_scope(&mut self, scope: Shared<Universe>) {
   |                                               ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-lang\src\trans\rust.rs:49:22
   |
49 | use crate::universe::Universe;
   |                      ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-lang\src\trans\rust.rs:69:43
   |
69 |     scope: Option<Shared<crate::universe::Universe>>,     // Old (deprecated)
   |                                           ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
    --> crates\auto-lang\src\trans\rust.rs:2767:31
     |
2767 | ) -> AutoResult<(Sink, Shared<Universe>)> {
     |                               ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
    --> crates\auto-lang\src\trans\rust.rs:2769:41
     |
2769 |     let scope = shared(crate::universe::Universe::default());
     |                                         ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
    --> crates\auto-lang\src\trans\rust.rs:2784:41
     |
2784 |     let scope = shared(crate::universe::Universe::default());
     |                                         ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-lang\src\trans\rust.rs:84:49
   |
84 |             scope: Some(shared(crate::universe::Universe::default())),  // Old (deprecated)
   |                                                 ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\trans\rust.rs:106:64
    |
106 |     pub fn set_scope(&mut self, scope: Shared<crate::universe::Universe>) {
    |                                                                ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
 --> crates\auto-lang\src\trans\python.rs:3:22
  |
3 | use crate::universe::Universe;
  |                      ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-lang\src\trans\python.rs:16:19
   |
16 |     scope: Shared<Universe>,
   |                   ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-lang\src\trans\python.rs:25:27
   |
25 |             scope: shared(Universe::default()),
   |                           ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-lang\src\trans\python.rs:29:47
   |
29 |     pub fn set_scope(&mut self, scope: Shared<Universe>) {
   |                                               ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
 --> crates\auto-lang\src\trans\javascript.rs:3:22
  |
3 | use crate::universe::Universe;
  |                      ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-lang\src\trans\javascript.rs:13:19
   |
13 |     scope: Shared<Universe>,
   |                   ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-lang\src\trans\javascript.rs:20:27
   |
20 |             scope: shared(Universe::default()),
   |                           ^^^^^^^^

warning: use of deprecated struct `universe::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-lang\src\trans\javascript.rs:24:47
   |
24 |     pub fn set_scope(&mut self, scope: Shared<Universe>) {
   |                                               ^^^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
  --> crates\auto-lang\src\vm.rs:54:44
   |
54 | pub type VmFunction = fn(&mut crate::eval::Evaler, auto_val::Value) -> auto_val::Value;
   |                                            ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
  --> crates\auto-lang\src\vm.rs:59:26
   |
59 |     fn(&mut crate::eval::Evaler, &mut auto_val::Value, Vec<auto_val::Value>) -> auto_val::Value;
   |                          ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
 --> crates\auto-lang\src\vm\builder.rs:2:24
  |
2 | use crate::{ast, eval::Evaler};
  |                        ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
  --> crates\auto-lang\src\vm\builder.rs:13:41
   |
13 | pub fn string_builder_new(_evaler: &mut Evaler, capacity: Value) -> Value {
   |                                         ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
  --> crates\auto-lang\src\vm\builder.rs:38:44
   |
38 | pub fn string_builder_append(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
   |                                            ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
  --> crates\auto-lang\src\vm\builder.rs:63:49
   |
63 | pub fn string_builder_append_char(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
   |                                                 ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
  --> crates\auto-lang\src\vm\builder.rs:89:48
   |
89 | pub fn string_builder_append_int(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
   |                                                ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\builder.rs:114:43
    |
114 | pub fn string_builder_build(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                           ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\builder.rs:135:43
    |
135 | pub fn string_builder_clear(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                           ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\builder.rs:157:41
    |
157 | pub fn string_builder_len(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                         ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\builder.rs:178:42
    |
178 | pub fn string_builder_drop(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                          ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\builder.rs:194:48
    |
194 | pub fn string_builder_new_static(_evaler: &mut Evaler, arg: Value) -> Value {
    |                                                ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\builder.rs:199:61
    |
199 | pub fn string_builder_new_with_default_static(_evaler: &mut Evaler, _arg: Value) -> Value {
    |                                                             ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
 --> crates\auto-lang\src\vm\collections.rs:5:24
  |
5 | use crate::{ast, eval::Evaler};
  |                        ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:207:35
    |
207 | pub fn hash_map_new(_evaler: &mut Evaler, _capacity: Value) -> Value {
    |                                   ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:229:42
    |
229 | pub fn hash_map_insert_str(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    |                                          ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:257:42
    |
257 | pub fn hash_map_insert_int(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    |                                          ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:262:39
    |
262 | pub fn hash_map_get_str(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    |                                       ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:286:39
    |
286 | pub fn hash_map_get_int(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    |                                       ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:291:40
    |
291 | pub fn hash_map_contains(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    |                                        ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:315:38
    |
315 | pub fn hash_map_remove(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    |                                      ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:340:36
    |
340 | pub fn hash_map_size(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                    ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:361:37
    |
361 | pub fn hash_map_clear(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                     ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:383:36
    |
383 | pub fn hash_map_drop(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                    ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:399:42
    |
399 | pub fn hash_map_new_static(_evaler: &mut Evaler, _arg: Value) -> Value {
    |                                          ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:430:35
    |
430 | pub fn hash_set_new(_evaler: &mut Evaler, _arg: Value) -> Value {
    |                                   ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:451:38
    |
451 | pub fn hash_set_insert(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    |                                      ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:476:40
    |
476 | pub fn hash_set_contains(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    |                                        ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:500:38
    |
500 | pub fn hash_set_remove(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    |                                      ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:525:36
    |
525 | pub fn hash_set_size(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                    ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:546:37
    |
546 | pub fn hash_set_clear(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                     ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:568:36
    |
568 | pub fn hash_set_drop(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                    ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:587:36
    |
587 | pub fn vec_deque_new(_evaler: &mut Evaler, _arg: Value) -> Value {
    |                                    ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:611:42
    |
611 | pub fn vec_deque_push_back(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    |                                          ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:635:43
    |
635 | pub fn vec_deque_push_front(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    |                                           ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:659:41
    |
659 | pub fn vec_deque_pop_back(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                         ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:680:42
    |
680 | pub fn vec_deque_pop_front(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                          ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:701:38
    |
701 | pub fn vec_deque_front(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                      ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:722:37
    |
722 | pub fn vec_deque_back(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                     ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:743:37
    |
743 | pub fn vec_deque_size(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                     ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:764:41
    |
764 | pub fn vec_deque_is_empty(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                         ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:785:38
    |
785 | pub fn vec_deque_clear(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                      ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:807:37
    |
807 | pub fn vec_deque_drop(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                     ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:826:36
    |
826 | pub fn btree_map_new(_evaler: &mut Evaler, _arg: Value) -> Value {
    |                                    ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:850:39
    |
850 | pub fn btree_map_insert(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    |                                       ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:876:36
    |
876 | pub fn btree_map_get(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    |                                    ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:900:41
    |
900 | pub fn btree_map_contains(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    |                                         ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:924:39
    |
924 | pub fn btree_map_remove(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    |                                       ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:949:37
    |
949 | pub fn btree_map_size(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                     ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:970:41
    |
970 | pub fn btree_map_is_empty(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                         ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\collections.rs:991:38
    |
991 | pub fn btree_map_clear(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                      ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\vm\collections.rs:1013:42
     |
1013 | pub fn btree_map_first_key(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
     |                                          ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\vm\collections.rs:1036:41
     |
1036 | pub fn btree_map_last_key(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
     |                                         ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\vm\collections.rs:1059:37
     |
1059 | pub fn btree_map_drop(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
     |                                     ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
 --> crates\auto-lang\src\vm\io.rs:7:24
  |
7 | use crate::{ast, eval::Evaler};
  |                        ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
 --> crates\auto-lang\src\vm\io.rs:9:27
  |
9 | pub fn open(_evaler: &mut Evaler, path: Value) -> Value {
  |                           ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
  --> crates\auto-lang\src\vm\io.rs:59:32
   |
59 | pub fn read_text(_evaler: &mut Evaler, file: &mut Value) -> Value {
   |                                ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
  --> crates\auto-lang\src\vm\io.rs:83:32
   |
83 | pub fn read_line(_evaler: &mut Evaler, file: &mut Value) -> Value {
   |                                ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\io.rs:119:28
    |
119 | pub fn close(_evaler: &mut Evaler, file: &mut Value) -> Value {
    |                            ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\io.rs:134:39
    |
134 | pub fn read_text_method(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                       ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\io.rs:139:35
    |
139 | pub fn close_method(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                   ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\io.rs:143:39
    |
143 | pub fn read_line_method(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                       ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\io.rs:147:32
    |
147 | pub fn read_char(_evaler: &mut Evaler, file: &mut Value) -> Value {
    |                                ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\io.rs:173:39
    |
173 | pub fn read_char_method(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                       ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\io.rs:177:31
    |
177 | pub fn read_buf(_evaler: &mut Evaler, _file: &mut Value, _buf: &mut Value, _size: i64) -> Value {
    |                               ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\io.rs:182:38
    |
182 | pub fn read_buf_method(_evaler: &mut Evaler, _instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                      ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\io.rs:187:33
    |
187 | pub fn write_line(_evaler: &mut Evaler, file: &mut Value, line: &str) -> Value {
    |                                 ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\io.rs:212:40
    |
212 | pub fn write_line_method(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    |                                        ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\io.rs:225:28
    |
225 | pub fn flush(_evaler: &mut Evaler, file: &mut Value) -> Value {
    |                            ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\io.rs:250:35
    |
250 | pub fn flush_method(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                   ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
 --> crates\auto-lang\src\vm\list.rs:6:24
  |
6 | use crate::{ast, eval::Evaler};
  |                        ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
  --> crates\auto-lang\src\vm\list.rs:12:38
   |
12 | pub fn list_new_static(_evaler: &mut Evaler, _arg: Value) -> Value {
   |                                      ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
  --> crates\auto-lang\src\vm\list.rs:22:31
   |
22 | pub fn list_new(_evaler: &mut Evaler, initial: Value) -> Value {
   |                               ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
  --> crates\auto-lang\src\vm\list.rs:63:32
   |
63 | pub fn list_push(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
   |                                ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
  --> crates\auto-lang\src\vm\list.rs:87:31
   |
87 | pub fn list_pop(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
   |                               ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\list.rs:109:31
    |
109 | pub fn list_len(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                               ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\list.rs:131:36
    |
131 | pub fn list_is_empty(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                    ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\list.rs:154:36
    |
154 | pub fn list_capacity(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                    ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\list.rs:176:33
    |
176 | pub fn list_clear(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                 ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\list.rs:198:35
    |
198 | pub fn list_reserve(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    |                                   ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\list.rs:224:31
    |
224 | pub fn list_get(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    |                               ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\list.rs:252:31
    |
252 | pub fn list_set(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    |                               ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\list.rs:280:34
    |
280 | pub fn list_insert(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    |                                  ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\list.rs:307:34
    |
307 | pub fn list_remove(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    |                                  ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\list.rs:335:32
    |
335 | pub fn list_iter(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\list.rs:367:32
    |
367 | pub fn list_drop(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\list.rs:389:37
    |
389 | pub fn list_iter_next(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                     ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\list.rs:435:36
    |
435 | pub fn list_iter_map(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    |                                    ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\list.rs:485:36
    |
485 | pub fn map_iter_next(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                    ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\list.rs:599:39
    |
599 | pub fn list_iter_filter(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    |                                       ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\list.rs:648:39
    |
648 | pub fn filter_iter_next(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                       ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\list.rs:765:39
    |
765 | pub fn list_iter_reduce(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    |                                       ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\list.rs:864:38
    |
864 | pub fn list_iter_count(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                      ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\list.rs:928:41
    |
928 | pub fn list_iter_for_each(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
    |                                         ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\list.rs:975:40
    |
975 | pub fn list_iter_collect(_evaler: &mut Evaler, instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                        ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\vm\list.rs:1065:36
     |
1065 | pub fn list_iter_any(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
     |                                    ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\vm\list.rs:1127:36
     |
1127 | pub fn list_iter_all(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
     |                                    ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\vm\list.rs:1181:37
     |
1181 | pub fn list_iter_find(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
     |                                     ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\vm\list.rs:1238:38
     |
1238 | pub fn filter_iter_map(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
     |                                      ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\vm\list.rs:1291:38
     |
1291 | pub fn map_iter_filter(_evaler: &mut Evaler, instance: &mut Value, args: Vec<Value>) -> Value {
     |                                      ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
 --> crates\auto-lang\src\vm\memory.rs:1:18
  |
1 | use crate::eval::Evaler;
  |                  ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
 --> crates\auto-lang\src\vm\memory.rs:7:34
  |
7 | pub fn alloc_array(_evaler: &mut Evaler, size_val: Value) -> Value {
  |                                  ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
  --> crates\auto-lang\src\vm\memory.rs:23:36
   |
23 | pub fn realloc_array(_evaler: &mut Evaler, array: Value, size_val: Value) -> Value {
   |                                    ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
  --> crates\auto-lang\src\vm\memory.rs:43:33
   |
43 | pub fn free_array(_evaler: &mut Evaler, _array: Value) -> Value {
   |                                 ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
  --> crates\auto-lang\src\vm\memory.rs:50:43
   |
50 | pub fn realloc_array_wrapped(evaler: &mut Evaler, args: Value) -> Value {
   |                                           ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
 --> crates\auto-lang\src\vm\storage.rs:5:18
  |
5 | use crate::eval::Evaler;
  |                  ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
  --> crates\auto-lang\src\vm\storage.rs:13:31
   |
13 | pub fn heap_new(_evaler: &mut Evaler, _args: Value) -> Value {
   |                               ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
  --> crates\auto-lang\src\vm\storage.rs:33:32
   |
33 | pub fn heap_data(_evaler: &mut Evaler, self_instance: &mut Value, _args: Vec<Value>) -> Value {
   |                                ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
  --> crates\auto-lang\src\vm\storage.rs:44:36
   |
44 | pub fn heap_capacity(_evaler: &mut Evaler, self_instance: &mut Value, _args: Vec<Value>) -> Value {
   |                                    ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
  --> crates\auto-lang\src\vm\storage.rs:56:36
   |
56 | pub fn heap_try_grow(_evaler: &mut Evaler, self_instance: &mut Value, args: Vec<Value>) -> Value {
   |                                    ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\storage.rs:123:32
    |
123 | pub fn heap_drop(_evaler: &mut Evaler, self_instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\storage.rs:164:39
    |
164 | pub fn inline_int64_new(_evaler: &mut Evaler, _args: Value) -> Value {
    |                                       ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\storage.rs:182:40
    |
182 | pub fn inline_int64_data(_evaler: &mut Evaler, self_instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                        ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\storage.rs:194:44
    |
194 | pub fn inline_int64_capacity(_evaler: &mut Evaler, _self_instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                            ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\storage.rs:200:44
    |
200 | pub fn inline_int64_try_grow(_evaler: &mut Evaler, _self_instance: &mut Value, args: Vec<Value>) -> Value {
    |                                            ^^^^^^

warning: use of deprecated struct `eval::Evaler`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\vm\storage.rs:217:40
    |
217 | pub fn inline_int64_drop(_evaler: &mut Evaler, _self_instance: &mut Value, _args: Vec<Value>) -> Value {
    |                                        ^^^^^^

warning: use of deprecated field `interp::Interpreter::result`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\atom.rs:758:45
    |
758 |         let result = std::mem::replace(&mut self.interp.result, auto_val::Value::Nil);
    |                                             ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::result`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
  --> crates\auto-lang\src\execution_engine.rs:67:8
   |
67 |     Ok(interpreter.result.repr().to_string())
   |        ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `universe::Universe::cur_spot`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\eval.rs:110:29
    |
110 |         let current_scope = universe.borrow().cur_spot.clone();
    |                             ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::db`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:114:13
    |
114 |             db: None,
    |             ^^^^^^^^

warning: use of deprecated field `eval::Evaler::engine`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:115:13
    |
115 |             engine: None,
    |             ^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::current_scope`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:117:13
    |
117 |             current_scope,
    |             ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:119:13
    |
119 |             universe,
    |             ^^^^^^^^

warning: use of deprecated field `eval::Evaler::tempo_for_nodes`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:120:13
    |
120 |             tempo_for_nodes: HashMap::new(),
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::mode`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:121:13
    |
121 |             mode: EvalMode::SCRIPT,
    |             ^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::skip_check`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:122:13
    |
122 |             skip_check: false,
    |             ^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::borrow_checker`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:123:13
    |
123 |             borrow_checker: crate::ownership::borrow::BorrowChecker::new(),
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::lifetime_ctx`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:124:13
    |
124 |             lifetime_ctx: crate::ownership::lifetime::LifetimeContext::new(),
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::closures`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:125:13
    |
125 |             closures: HashMap::new(),
    |             ^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::next_closure_id`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:126:13
    |
126 |             next_closure_id: 0,
    |             ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::lib_paths`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:127:13
    |
127 |             lib_paths: Vec::new(),
    |             ^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:144:13
    |
144 |             self.universe.borrow_mut().set_evaluator_raw(eval_ptr);
    |             ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::mode`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:149:9
    |
149 |         self.mode = mode;
    |         ^^^^^^^^^

warning: use of deprecated field `eval::Evaler::mode`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:154:9
    |
154 |         self.mode = mode;
    |         ^^^^^^^^^

warning: use of deprecated field `eval::Evaler::tempo_for_nodes`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:158:9
    |
158 |         self.tempo_for_nodes.insert(name.into(), tempo);
    |         ^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::skip_check`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:162:9
    |
162 |         self.skip_check = true;
    |         ^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::lib_paths`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:167:9
    |
167 |         self.lib_paths = paths;
    |         ^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::lib_paths`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:178:9
    |
178 |         self.lib_paths.push(path);
    |         ^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::lib_paths`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:183:10
    |
183 |         &self.lib_paths
    |          ^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:208:10
    |
208 |         &self.universe
    |          ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:221:14
    |
221 |         &mut self.universe
    |              ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::db`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:229:9
    |
229 |         self.db = Some(db);
    |         ^^^^^^^

warning: use of deprecated field `eval::Evaler::engine`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:236:9
    |
236 |         self.engine = Some(engine);
    |         ^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::db`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:247:9
    |
247 |         self.db.as_ref()
    |         ^^^^^^^

warning: use of deprecated field `eval::Evaler::engine`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:257:9
    |
257 |         self.engine.as_ref()
    |         ^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::db`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:282:28
    |
282 |         if let Some(db) = &self.db {
    |                            ^^^^^^^

warning: use of deprecated field `eval::Evaler::current_scope`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:286:54
    |
286 |             let new_sid = crate::scope::Sid::kid_of(&self.current_scope, "_block");
    |                                                      ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::engine`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:295:36
    |
295 |             if let Some(engine) = &self.engine {
    |                                    ^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::current_scope`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:300:13
    |
300 |             self.current_scope = new_sid;
    |             ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:303:13
    |
303 |             self.universe.borrow_mut().enter_scope();
    |             ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::current_scope`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:305:13
    |
305 |             self.current_scope = self.universe.borrow().cur_spot.clone();
    |             ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:305:34
    |
305 |             self.current_scope = self.universe.borrow().cur_spot.clone();
    |                                  ^^^^^^^^^^^^^

warning: use of deprecated field `universe::Universe::cur_spot`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\eval.rs:305:34
    |
305 |             self.current_scope = self.universe.borrow().cur_spot.clone();
    |                                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::engine`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:326:32
    |
326 |         if let Some(engine) = &self.engine {
    |                                ^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::current_scope`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:330:39
    |
330 |                 if let Some(parent) = self.current_scope.parent() {
    |                                       ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::current_scope`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:331:21
    |
331 |                     self.current_scope = parent;
    |                     ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::current_scope`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:334:21
    |
334 |                     self.current_scope = crate::scope::SID_PATH_GLOBAL.clone();
    |                     ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:339:13
    |
339 |             self.universe.borrow_mut().exit_scope();
    |             ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::current_scope`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:341:13
    |
341 |             self.current_scope = self.universe.borrow().cur_spot.clone();
    |             ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:341:34
    |
341 |             self.current_scope = self.universe.borrow().cur_spot.clone();
    |                                  ^^^^^^^^^^^^^

warning: use of deprecated field `universe::Universe::cur_spot`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-lang\src\eval.rs:341:34
    |
341 |             self.current_scope = self.universe.borrow().cur_spot.clone();
    |                                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::db`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:361:28
    |
361 |         if let Some(db) = &self.db {
    |                            ^^^^^^^

warning: use of deprecated field `eval::Evaler::current_scope`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:365:39
    |
365 |             let mut search_sid = Some(self.current_scope.clone());
    |                                       ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:384:13
    |
384 |             self.universe.borrow().lookup_meta(name)
    |             ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:387:13
    |
387 |             self.universe.borrow().lookup_meta(name)
    |             ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::engine`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:407:32
    |
407 |         if let Some(engine) = &self.engine {
    |                                ^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:416:17
    |
416 |                 self.universe.borrow().lookup_val(name)
    |                 ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:420:13
    |
420 |             self.universe.borrow().lookup_val(name)
    |             ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:444:9
    |
444 |         self.universe.borrow_mut().set_local_val(name, value);
    |         ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:475:9
    |
475 |         self.universe.borrow_mut().define(name, meta);
    |         ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:493:9
    |
493 |         self.universe.borrow_mut().remove_local(name)
    |         ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:511:9
    |
511 |         self.universe.borrow_mut().set_global(name, value);
    |         ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:546:9
    |
546 |         self.universe.borrow_mut().define_type(name, meta);
    |         ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::engine`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:587:32
    |
587 |         if let Some(engine) = &self.engine {
    |                                ^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:590:13
    |
590 |             self.universe.borrow_mut().add_vmref(data)
    |             ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::engine`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:599:32
    |
599 |         if let Some(engine) = &self.engine {
    |                                ^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:602:13
    |
602 |             self.universe.borrow_mut().drop_vmref(id);
    |             ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:614:9
    |
614 |         self.universe.borrow().has_local(name)
    |         ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:621:9
    |
621 |         self.universe.borrow_mut().clear_moved(name);
    |         ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:628:9
    |
628 |         self.universe.borrow().exists(name)
    |         ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:635:9
    |
635 |         self.universe.borrow_mut().update_val(name, value);
    |         ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:642:9
    |
642 |         self.universe.borrow().get_defined_names()
    |         ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:649:9
    |
649 |         self.universe.borrow().has_arg(name)
    |         ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:656:9
    |
656 |         self.universe.borrow().get_arg(name)
    |         ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:663:9
    |
663 |         self.universe.borrow_mut().alloc_value(data)
    |         ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:670:9
    |
670 |         self.universe.borrow().deref_val(val)
    |         ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:677:9
    |
677 |         self.universe.borrow().lookup_type(name)
    |         ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:684:9
    |
684 |         self.universe.borrow_mut().mark_moved(name);
    |         ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:691:9
    |
691 |         self.universe.borrow_mut().enter_fn(name);
    |         ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:698:9
    |
698 |         self.universe.borrow_mut().set_local_obj(obj);
    |         ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:705:9
    |
705 |         self.universe.borrow_mut().register_spec(spec);
    |         ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:712:9
    |
712 |         self.universe.borrow().get_spec(name)
    |         ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::mode`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
   --> crates\auto-lang\src\eval.rs:720:15
    |
720 |         match self.mode {
    |               ^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:1000:9
     |
1000 |         self.universe.borrow().dump();
     |         ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::lib_paths`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:1071:78
     |
1071 |         eprintln!("Searching for module '{}', lib_paths: {:?}", module_name, self.lib_paths);
     |                                                                              ^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::lib_paths`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:1078:25
     |
1078 |         for lib_dir in &self.lib_paths {
     |                         ^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::mode`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:1256:25
     |
1256 |         let res = match self.mode {
     |                         ^^^^^^^^^

warning: use of deprecated field `eval::Evaler::mode`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:1292:18
     |
1292 |         Ok(match self.mode {
     |                  ^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:1327:37
     |
1327 | ...                   self.universe
     |                       ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::mode`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:1484:29
     |
1484 |             return Ok(match self.mode {
     |                             ^^^^^^^^^

warning: use of deprecated field `eval::Evaler::mode`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:1552:29
     |
1552 |             return Ok(match self.mode {
     |                             ^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:1674:43
     |
1674 | ...                   let uni = self.universe.borrow();
     |                                 ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::mode`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:1714:45
     |
1714 | ...                   return Ok(match self.mode {
     |                                       ^^^^^^^^^

warning: use of deprecated field `eval::Evaler::mode`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:1746:22
     |
1746 |             Ok(match self.mode {
     |                      ^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:2084:31
     |
2084 |                           match self
     |  _______________________________^
2085 | |                             .universe
     | |_____________________________________^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:2110:39
     |
2110 |   ...                   match self
     |  _____________________________^
2111 | | ...                       .universe
     | |___________________________________^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:2148:35
     |
2148 |   ...                   match self
     |  _____________________________^
2149 | | ...                       .universe
     | |___________________________________^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:2182:43
     |
2182 |   ...                   match self
     |  _____________________________^
2183 | | ...                       .universe
     | |___________________________________^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:2213:39
     |
2213 |   ...                   match self
     |  _____________________________^
2214 | | ...                       .universe
     | |___________________________________^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:2258:51
     |
2258 |   ...                   match self
     |  _____________________________^
2259 | | ...                       .universe
     | |___________________________________^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:2280:43
     |
2280 |   ...                   match self
     |  _____________________________^
2281 | | ...                       .universe
     | |___________________________________^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:2319:47
     |
2319 |   ...                   match self
     |  _____________________________^
2320 | | ...                       .universe
     | |___________________________________^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:2366:43
     |
2366 |   ...                   match self
     |  _____________________________^
2367 | | ...                       .universe
     | |___________________________________^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:2401:39
     |
2401 |   ...                   match self
     |  _____________________________^
2402 | | ...                       .universe
     | |___________________________________^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:2434:47
     |
2434 |   ...                   match self
     |  _____________________________^
2435 | | ...                       .universe
     | |___________________________________^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:2470:43
     |
2470 |   ...                   match self
     |  _____________________________^
2471 | | ...                       .universe
     | |___________________________________^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:2520:47
     |
2520 |   ...                   match self
     |  _____________________________^
2521 | | ...                       .universe
     | |___________________________________^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:2558:47
     |
2558 |   ...                   match self
     |  _____________________________^
2559 | | ...                       .universe
     | |___________________________________^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:2609:51
     |
2609 |   ...                   match self
     |  _____________________________^
2610 | | ...                       .universe
     | |___________________________________^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:2863:9
     |
2863 |         self.universe.borrow().lookup_val_id(name)
     |         ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:2869:37
     |
2869 |             Value::ValueRef(vid) => self.universe.borrow().get_value(*vid),
     |                                     ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:2877:37
     |
2877 |               Value::ValueRef(vid) => self
     |  _____________________________________^
2878 | |                 .universe
     | |_________________________^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:3148:25
     |
3148 |                         self.universe
     |                         ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:3720:33
     |
3720 |                   let method_fn = self
     |  _________________________________^
3721 | |                     .universe
     | |_____________________________^

warning: use of deprecated field `universe::Universe::types`: Use Database + ExecutionEngine instead (see Plan 064)
    --> crates\auto-lang\src\eval.rs:3720:33
     |
3720 |                   let method_fn = self
     |  _________________________________^
3721 | |                     .universe
3722 | |                     .borrow()
3723 | |                     .types
     | |__________________________^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:3742:33
     |
3742 |                   let method_fn = self
     |  _________________________________^
3743 | |                     .universe
     | |_____________________________^

warning: use of deprecated field `universe::Universe::types`: Use Database + ExecutionEngine instead (see Plan 064)
    --> crates\auto-lang\src\eval.rs:3742:33
     |
3742 |                   let method_fn = self
     |  _________________________________^
3743 | |                     .universe
3744 | |                     .borrow()
3745 | |                     .types
     | |__________________________^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:3755:25
     |
3755 |                         self.universe
     |                         ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:3897:33
     |
3897 |                   let method_fn = self
     |  _________________________________^
3898 | |                     .universe
     | |_____________________________^

warning: use of deprecated field `universe::Universe::types`: Use Database + ExecutionEngine instead (see Plan 064)
    --> crates\auto-lang\src\eval.rs:3897:33
     |
3897 |                   let method_fn = self
     |  _________________________________^
3898 | |                     .universe
3899 | |                     .borrow()
3900 | |                     .types
     | |__________________________^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:3915:27
     |
3915 |           let fn_decl_opt = self
     |  ___________________________^
3916 | |             .universe
     | |_____________________^

warning: use of deprecated field `eval::Evaler::lifetime_ctx`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:4532:32
     |
4532 |                 let lifetime = self.lifetime_ctx.fresh_lifetime();
     |                                ^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::borrow_checker`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:4535:35
     |
4535 |                 if let Err(err) = self.borrow_checker.check_borrow(
     |                                   ^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::lifetime_ctx`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:4554:32
     |
4554 |                 let lifetime = self.lifetime_ctx.fresh_lifetime();
     |                                ^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::borrow_checker`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:4557:35
     |
4557 |                 if let Err(err) = self.borrow_checker.check_borrow(
     |                                   ^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::lifetime_ctx`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:4576:32
     |
4576 |                 let lifetime = self.lifetime_ctx.fresh_lifetime();
     |                                ^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::borrow_checker`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:4579:35
     |
4579 |                 if let Err(err) = self.borrow_checker.check_borrow(
     |                                   ^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::lifetime_ctx`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:4606:32
     |
4606 |                 let lifetime = self.lifetime_ctx.fresh_lifetime();
     |                                ^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::borrow_checker`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:4607:35
     |
4607 |                 if let Err(err) = self.borrow_checker.check_borrow(
     |                                   ^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:4751:25
     |
4751 |                   let v = self
     |  _________________________^
4752 | |                     .universe
     | |_____________________________^

warning: use of deprecated field `eval::Evaler::skip_check`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:4760:20
     |
4760 |                 if self.skip_check {
     |                    ^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:4773:9
     |
4773 |         self.universe
     |         ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:4790:21
     |
4790 |                     self.universe
     |                     ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:4979:9
     |
4979 |         self.universe
     |         ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:5341:46
     |
5341 |                           let found_in_types = self
     |  ______________________________________________^
5342 | |                             .universe
     | |_____________________________________^

warning: use of deprecated field `universe::Universe::types`: Use Database + ExecutionEngine instead (see Plan 064)
    --> crates\auto-lang\src\eval.rs:5341:46
     |
5341 |                           let found_in_types = self
     |  ______________________________________________^
5342 | |                             .universe
5343 | |                             .borrow()
5344 | |                             .types
     | |__________________________________^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:5358:49
     |
5358 |   ...                   let method_exists = self
     |  ___________________________________________^
5359 | | ...                       .universe
     | |___________________________________^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:5388:28
     |
5388 |           let is_mid_value = self
     |  ____________________________^
5389 | |             .universe
     | |_____________________^

warning: use of deprecated field `eval::Evaler::tempo_for_nodes`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:5553:21
     |
5553 |           let tempo = self
     |  _____________________^
5554 | |             .tempo_for_nodes
     | |____________________________^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:5566:29
     |
5566 | ...                   self.universe
     |                       ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:5584:29
     |
5584 | ...                   self.universe
     |                       ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::universe`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:5629:17
     |
5629 |                 self.universe
     |                 ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::next_closure_id`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:5852:26
     |
5852 |         let closure_id = self.next_closure_id;
     |                          ^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::next_closure_id`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:5853:9
     |
5853 |         self.next_closure_id += 1;
     |         ^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::closures`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:5864:9
     |
5864 |         self.closures.insert(closure_id, eval_closure);
     |         ^^^^^^^^^^^^^

warning: use of deprecated field `eval::Evaler::closures`: Use AutoVM instead (Plan 068 Phase 9). See run() or run_bigvm() functions.
    --> crates\auto-lang\src\eval.rs:5879:28
     |
5879 |           let eval_closure = self
     |  ____________________________^
5880 | |             .closures
     | |_____________________^

warning: use of deprecated field `interp::Interpreter::session`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
  --> crates\auto-lang\src\interp.rs:91:13
   |
91 |             session,
   |             ^^^^^^^

warning: use of deprecated field `interp::Interpreter::engine`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
  --> crates\auto-lang\src\interp.rs:92:13
   |
92 |             engine,
   |             ^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
  --> crates\auto-lang\src\interp.rs:93:13
   |
93 |             evaler: Evaler::new(scope.clone()),
   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::scope`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
  --> crates\auto-lang\src\interp.rs:94:13
   |
94 |             scope,
   |             ^^^^^

warning: use of deprecated field `interp::Interpreter::result`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
  --> crates\auto-lang\src\interp.rs:95:13
   |
95 |             result: Value::Nil,
   |             ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::fstr_note`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
  --> crates\auto-lang\src\interp.rs:96:13
   |
96 |             fstr_note: '$',
   |             ^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::skip_check`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
  --> crates\auto-lang\src\interp.rs:97:13
   |
97 |             skip_check: false,
   |             ^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::enable_error_recovery`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
  --> crates\auto-lang\src\interp.rs:98:13
   |
98 |             enable_error_recovery: false,
   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::lib_paths`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
  --> crates\auto-lang\src\interp.rs:99:13
   |
99 |             lib_paths: Vec::new(),
   |             ^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:103:9
    |
103 |         interpreter.evaler.register_with_universe();
    |         ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:106:9
    |
106 |         interpreter.evaler.set_db(db_ref);
    |         ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:107:9
    |
107 |         interpreter.evaler.set_engine(engine_ref.clone());
    |         ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:112:52
    |
112 |         engine_ref.borrow_mut().set_evaluator(&mut interpreter.evaler);
    |                                                    ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::scope`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:117:9
    |
117 |         interpreter.scope.borrow_mut().inject_environment(target);
    |         ^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::scope`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:126:34
    |
126 |         Self::load_stdlib_types(&interpreter.scope);
    |                                  ^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::fstr_note`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:276:9
    |
276 |         self.fstr_note = note;
    |         ^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::session`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:290:13
    |
290 |             session,
    |             ^^^^^^^

warning: use of deprecated field `interp::Interpreter::engine`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:291:13
    |
291 |             engine,
    |             ^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:292:13
    |
292 |             evaler: Evaler::new(univ.clone()),
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::scope`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:293:13
    |
293 |             scope: univ.clone(),
    |             ^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::fstr_note`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:294:13
    |
294 |             fstr_note: '$',
    |             ^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::result`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:295:13
    |
295 |             result: Value::Nil,
    |             ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::skip_check`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:296:13
    |
296 |             skip_check: false,
    |             ^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::enable_error_recovery`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:297:13
    |
297 |             enable_error_recovery: false,
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::lib_paths`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:298:13
    |
298 |             lib_paths: Vec::new(),
    |             ^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:302:9
    |
302 |         interp.evaler.register_with_universe();
    |         ^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:305:9
    |
305 |         interp.evaler.set_db(db_ref);
    |         ^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:306:9
    |
306 |         interp.evaler.set_engine(engine_ref.clone());
    |         ^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:309:52
    |
309 |         engine_ref.borrow_mut().set_evaluator(&mut interp.evaler);
    |                                                    ^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::scope`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:313:34
    |
313 |         Self::load_stdlib_types(&interp.scope);
    |                                  ^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::session`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:342:13
    |
342 |             session: session_clone,
    |             ^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::engine`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:343:13
    |
343 |             engine,
    |             ^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:344:13
    |
344 |             evaler: Evaler::new(scope.clone()),
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::scope`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:345:13
    |
345 |             scope,
    |             ^^^^^

warning: use of deprecated field `interp::Interpreter::result`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:346:13
    |
346 |             result: Value::Nil,
    |             ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::fstr_note`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:347:13
    |
347 |             fstr_note: '$',
    |             ^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::skip_check`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:348:13
    |
348 |             skip_check: false,
    |             ^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::enable_error_recovery`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:349:13
    |
349 |             enable_error_recovery: false,
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::lib_paths`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:350:13
    |
350 |             lib_paths: Vec::new(),
    |             ^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:354:9
    |
354 |         interpreter.evaler.register_with_universe();
    |         ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:357:9
    |
357 |         interpreter.evaler.set_db(db_ref);
    |         ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:358:9
    |
358 |         interpreter.evaler.set_engine(engine_ref.clone());
    |         ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:361:52
    |
361 |         engine_ref.borrow_mut().set_evaluator(&mut interpreter.evaler);
    |                                                    ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::scope`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:365:9
    |
365 |         interpreter.scope.borrow_mut().inject_environment(target);
    |         ^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::scope`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:374:34
    |
374 |         Self::load_stdlib_types(&interpreter.scope);
    |                                  ^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::session`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:436:13
    |
436 |             session: session_clone,
    |             ^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::engine`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:437:13
    |
437 |             engine,
    |             ^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:438:13
    |
438 |             evaler: Evaler::new(scope.clone()),
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::scope`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:439:13
    |
439 |             scope,
    |             ^^^^^

warning: use of deprecated field `interp::Interpreter::result`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:440:13
    |
440 |             result: Value::Nil,
    |             ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::fstr_note`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:441:13
    |
441 |             fstr_note: '$',
    |             ^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::skip_check`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:442:13
    |
442 |             skip_check: false,
    |             ^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::enable_error_recovery`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:443:13
    |
443 |             enable_error_recovery: false,
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::lib_paths`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:444:13
    |
444 |             lib_paths: Vec::new(),
    |             ^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:448:9
    |
448 |         interpreter.evaler.register_with_universe();
    |         ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:451:9
    |
451 |         interpreter.evaler.set_db(db_ref);
    |         ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:452:9
    |
452 |         interpreter.evaler.set_engine(engine_ref.clone());
    |         ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:455:52
    |
455 |         engine_ref.borrow_mut().set_evaluator(&mut interpreter.evaler);
    |                                                    ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::scope`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:459:9
    |
459 |         interpreter.scope.borrow_mut().inject_environment(target);
    |         ^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:474:9
    |
474 |         self.evaler = self.evaler.with_mode(mode);
    |         ^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:474:23
    |
474 |         self.evaler = self.evaler.with_mode(mode);
    |                       ^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::skip_check`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:484:9
    |
484 |         self.skip_check = true;
    |         ^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:485:9
    |
485 |         self.evaler.skip_check();
    |         ^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::enable_error_recovery`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:494:9
    |
494 |         self.enable_error_recovery = true;
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:504:9
    |
504 |         self.evaler.set_lib_paths(paths.clone());
    |         ^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::lib_paths`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:505:9
    |
505 |         self.lib_paths = paths;
    |         ^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::lib_paths`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:517:9
    |
517 |         self.lib_paths.push(path.clone());
    |         ^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:518:9
    |
518 |         self.evaler.add_lib_path(path);
    |         ^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::lib_paths`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:523:10
    |
523 |         &self.lib_paths
    |          ^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::fstr_note`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:529:29
    |
529 |         lexer.set_fstr_note(self.fstr_note);
    |                             ^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::scope`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:539:13
    |
539 |             self.scope.clone(),
    |             ^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::fstr_note`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:540:13
    |
540 |             self.fstr_note,
    |             ^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::skip_check`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:544:12
    |
544 |         if self.skip_check {
    |            ^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:548:22
    |
548 |         let result = self.evaler.eval(&ast)?;
    |                      ^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::scope`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:553:23
    |
553 |         let derefed = self.scope.borrow().deref_val(result);
    |                       ^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::result`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:554:9
    |
554 |         self.result = derefed;
    |         ^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::fstr_note`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:559:44
    |
559 |         self.eval_template_with_note(code, self.fstr_note)
    |                                            ^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:567:9
    |
567 |         self.evaler.set_mode(EvalMode::TEMPLATE);
    |         ^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::scope`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:571:66
    |
571 |         let mut parser = Parser::new_with_note(flipped.as_str(), self.scope.clone(), note);
    |                                                                  ^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:586:22
    |
586 |         let result = self.evaler.eval(&ast)?;
    |                      ^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::result`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:594:12
    |
594 |         Ok(self.result.clone())
    |            ^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::result`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:600:9
    |
600 |         self.result = self.eval_config(&code)?;
    |         ^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::scope`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:601:9
    |
601 |         self.scope
    |         ^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::result`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:603:35
    |
603 |             .set_global("result", self.result.clone());
    |                                   ^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::result`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:604:12
    |
604 |         Ok(self.result.clone())
    |            ^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::scope`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:608:54
    |
608 |         let mut parser = Parser::new_with_note(code, self.scope.clone(), self.fstr_note);
    |                                                      ^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::fstr_note`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:608:74
    |
608 |         let mut parser = Parser::new_with_note(code, self.scope.clone(), self.fstr_note);
    |                                                                          ^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::scope`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:610:45
    |
610 |         let mut config_evaler = Evaler::new(self.scope.clone()).with_mode(EvalMode::CONFIG);
    |                                             ^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::session`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:614:30
    |
614 |         config_evaler.set_db(self.session.db());
    |                              ^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::scope`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:621:54
    |
621 |         let mut parser = Parser::new_with_note(code, self.scope.clone(), self.fstr_note);
    |                                                      ^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::fstr_note`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:621:74
    |
621 |         let mut parser = Parser::new_with_note(code, self.scope.clone(), self.fstr_note);
    |                                                                          ^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::evaler`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:627:33
    |
627 |                     val = match self.evaler.eval_stmt(&stmt) {
    |                                 ^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::scope`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\interp.rs:634:32
    |
634 |                     let data = self.scope.borrow().clone_value(id).unwrap();
    |                                ^^^^^^^^^^

warning: use of deprecated field `universe::Universe::cur_spot`: Use Database + ExecutionEngine instead (see Plan 064)
    --> crates\auto-lang\src\parser.rs:2819:28
     |
2819 |             let cur_spot = self.scope.borrow().cur_spot.clone();
     |                            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `universe::Universe::cur_spot`: Use Database + ExecutionEngine instead (see Plan 064)
    --> crates\auto-lang\src\parser.rs:2988:24
     |
2988 |         let cur_spot = self.scope.borrow().cur_spot.clone();
     |                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::session`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
  --> crates\auto-lang\src\repl.rs:39:23
   |
39 |         let session = temp_interpreter.session.clone();
   |                       ^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::engine`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
  --> crates\auto-lang\src\repl.rs:40:22
   |
40 |         let engine = temp_interpreter.engine.clone();
   |                      ^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::scope`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
  --> crates\auto-lang\src\repl.rs:41:21
   |
41 |         let scope = temp_interpreter.scope.clone();
   |                     ^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::result`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\repl.rs:224:47
    |
224 |                 let formatted = format_value(&interpreter.result, &interpreter.scope);
    |                                               ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::scope`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\repl.rs:224:68
    |
224 |                 let formatted = format_value(&interpreter.result, &interpreter.scope);
    |                                                                    ^^^^^^^^^^^^^^^^^

warning: use of deprecated field `universe::Universe::scopes`: Use Database + ExecutionEngine instead (see Plan 064)
    --> crates\auto-lang\src\trans\c.rs:3954:39
     |
3954 |             for (_sid, scope_data) in scope_borrowed.scopes.iter() {
     |                                       ^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `universe::Universe::code_paks`: Use Database + ExecutionEngine instead (see Plan 064)
    --> crates\auto-lang\src\trans\c.rs:4075:36
     |
4075 |     let paks = std::mem::take(&mut parser.scope.borrow_mut().code_paks);
     |                                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `universe::Universe::code_paks`: Use Database + ExecutionEngine instead (see Plan 064)
    --> crates\auto-lang\src\trans\c.rs:4098:5
     |
4098 |     parser.scope.borrow_mut().code_paks = paks;
     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::result`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\lib.rs:256:8
    |
256 |     Ok(interpreter.result.repr().to_string())
    |        ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::result`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\lib.rs:273:8
    |
273 |     Ok(interpreter.result.repr().to_string())
    |        ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::result`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\lib.rs:321:8
    |
321 |     Ok(interpreter.result.repr().to_string())
    |        ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::result`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\lib.rs:356:8
    |
356 |     Ok(interpreter.result.repr().to_string())
    |        ^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `interp::Interpreter::result`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-lang\src\lib.rs:411:5
    |
411 |     interpreter.result = result;
    |     ^^^^^^^^^^^^^^^^^^

warning: use of deprecated method `trans::c::CTrans::set_scope`: Use with_database() instead (Phase 066)
   --> crates\auto-lang\src\lib.rs:570:11
    |
570 |     trans.set_scope(parser.scope.clone());
    |           ^^^^^^^^^

warning: use of deprecated method `trans::rust::RustTrans::set_scope`: Use with_database() instead (Phase 066)
   --> crates\auto-lang\src\lib.rs:597:11
    |
597 |     trans.set_scope(parser.scope.clone());
    |           ^^^^^^^^^

warning: unused import: `crate::vm::heap_object::HeapObject`
    --> crates\auto-lang\src\vm\native.rs:1020:9
     |
1020 |     use crate::vm::heap_object::HeapObject;
     |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::vm::heap_object::HeapObject`
   --> crates\auto-lang\src\vm\native.rs:975:9
    |
975 |     use crate::vm::heap_object::HeapObject;
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::vm::heap_object::HeapObject`
   --> crates\auto-lang\src\vm\native.rs:998:9
    |
998 |     use crate::vm::heap_object::HeapObject;
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::vm::heap_object::HeapObject`
   --> crates\auto-lang\src\vm\native.rs:948:9
    |
948 |     use crate::vm::heap_object::HeapObject;
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::vm::heap_object::HeapObject`
   --> crates\auto-lang\src\vm\native.rs:924:9
    |
924 |     use crate::vm::heap_object::HeapObject;
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::vm::heap_object::HeapObject`
   --> crates\auto-lang\src\vm\native.rs:897:9
    |
897 |     use crate::vm::heap_object::HeapObject;
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::vm::heap_object::HeapObject`
   --> crates\auto-lang\src\vm\native.rs:876:9
    |
876 |     use crate::vm::heap_object::HeapObject;
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::vm::heap_object::HeapObject`
   --> crates\auto-lang\src\vm\native.rs:786:9
    |
786 |     use crate::vm::heap_object::HeapObject;
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::vm::heap_object::HeapObject`
   --> crates\auto-lang\src\vm\native.rs:738:9
    |
738 |     use crate::vm::heap_object::HeapObject;
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::vm::heap_object::HeapObject`
   --> crates\auto-lang\src\vm\native.rs:844:9
    |
844 |     use crate::vm::heap_object::HeapObject;
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::vm::heap_object::HeapObject`
   --> crates\auto-lang\src\vm\native.rs:455:9
    |
455 |     use crate::vm::heap_object::HeapObject;
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::vm::heap_object::HeapObject`
   --> crates\auto-lang\src\vm\native.rs:681:9
    |
681 |     use crate::vm::heap_object::HeapObject;
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::vm::heap_object::HeapObject`
   --> crates\auto-lang\src\vm\native.rs:386:9
    |
386 |     use crate::vm::heap_object::HeapObject;
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::vm::heap_object::HeapObject`
   --> crates\auto-lang\src\vm\native.rs:363:9
    |
363 |     use crate::vm::heap_object::HeapObject;
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::vm::heap_object::HeapObject`
   --> crates\auto-lang\src\vm\native.rs:340:9
    |
340 |     use crate::vm::heap_object::HeapObject;
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::vm::heap_object::HeapObject`
   --> crates\auto-lang\src\vm\native.rs:316:9
    |
316 |     use crate::vm::heap_object::HeapObject;
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::vm::heap_object::HeapObject`
   --> crates\auto-lang\src\vm\native.rs:295:9
    |
295 |     use crate::vm::heap_object::HeapObject;
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::vm::heap_object::HeapObject`
   --> crates\auto-lang\src\vm\native.rs:228:9
    |
228 |     use crate::vm::heap_object::HeapObject;
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::vm::heap_object::HeapObject`
   --> crates\auto-lang\src\vm\native.rs:273:9
    |
273 |     use crate::vm::heap_object::HeapObject;
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::vm::heap_object::HeapObject`
   --> crates\auto-lang\src\vm\native.rs:251:9
    |
251 |     use crate::vm::heap_object::HeapObject;
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::vm::heap_object::HeapObject`
   --> crates\auto-lang\src\vm\native.rs:206:9
    |
206 |     use crate::vm::heap_object::HeapObject;
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: variable does not need to be mutable
  --> crates\auto-lang\src\autovm_persistent.rs:64:13
   |
64 |         let mut codegen = Codegen::new();
   |             ----^^^^^^^
   |             |
   |             help: remove this `mut`
   |
   = note: `#[warn(unused_mut)]` (part of `#[warn(unused)]`) on by default

warning: variable does not need to be mutable
  --> crates\auto-lang\src\config.rs:29:17
   |
29 |     pub fn args(mut self, args: &Obj) -> Self {
   |                 ----^^^^
   |                 |
   |                 help: remove this `mut`

warning: unused variable: `use_stmt`
    --> crates\auto-lang\src\eval.rs:1099:61
     |
1099 |     fn load_at_file(&mut self, file_path: &std::path::Path, use_stmt: &ast::Use) -> Value {
     |                                                             ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_use_stmt`
     |
     = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

warning: unused variable: `inst_data`
    --> crates\auto-lang\src\eval.rs:3123:40
     |
3123 |                 if let Value::Instance(ref inst_data) = &inst {
     |                                        ^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_inst_data`

warning: unreachable pattern
    --> crates\auto-lang\src\eval.rs:4514:13
     |
4395 |             Expr::I64(value) => Value::Int(*value as i32),  // i64 transpiles to Int for now
     |             ---------------- matches all the relevant values
...
4514 |             Expr::I64(value) => Value::I64(*value),
     |             ^^^^^^^^^^^^^^^^ no value can reach this
     |
     = note: `#[warn(unreachable_patterns)]` (part of `#[warn(unused)]`) on by default

warning: unused variable: `count`
   --> crates\auto-lang\src\interp.rs:577:66
    |
577 |                 if let crate::error::AutoError::MultipleErrors { count, plural, errors } = &e {
    |                                                                  ^^^^^ help: try ignoring the field: `count: _`

warning: unused variable: `plural`
   --> crates\auto-lang\src\interp.rs:577:73
    |
577 |                 if let crate::error::AutoError::MultipleErrors { count, plural, errors } = &e {
    |                                                                         ^^^^^^ help: try ignoring the field: `plural: _`

warning: unused variable: `ident`
    --> crates\auto-lang\src\parser.rs:1920:21
     |
1920 |                 let ident = Expr::Ident(name.clone());
     |                     ^^^^^ help: if this is intentional, prefix it with an underscore: `_ident`

warning: variable does not need to be mutable
    --> crates\auto-lang\src\parser.rs:3987:13
     |
3987 |         let mut is_rs = has_rs;
     |             ----^^^^^
     |             |
     |             help: remove this `mut`

warning: unused variable: `is_rs`
    --> crates\auto-lang\src\parser.rs:3987:13
     |
3987 |         let mut is_rs = has_rs;
     |             ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_is_rs`

warning: unused variable: `is_rs`
    --> crates\auto-lang\src\parser.rs:4204:13
     |
4204 |         let is_rs = has_rs;
     |             ^^^^^ help: if this is intentional, prefix it with an underscore: `_is_rs`

warning: variable does not need to be mutable
  --> crates\auto-lang\src\repl.rs:36:13
   |
36 |         let mut temp_interpreter = interp::Interpreter::new();
   |             ----^^^^^^^^^^^^^^^^
   |             |
   |             help: remove this `mut`

warning: unused variable: `db`
   --> crates\auto-lang\src\trans\rust.rs:173:28
    |
173 |         } else if let Some(db) = &self.db {
    |                            ^^ help: if this is intentional, prefix it with an underscore: `_db`

warning: unused variable: `class_type`
   --> crates\auto-lang\src\vm\codegen.rs:405:35
    |
405 |                         if let Ok(class_type) = self.generic_registry.get_or_create_type(
    |                                   ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_class_type`

warning: unused variable: `type_info`
   --> crates\auto-lang\src\vm\codegen.rs:497:45
    |
497 | ...                   if let Some(type_info) = self.get_type(type_name) {
    |                                   ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_type_info`

warning: unused variable: `enum_decl`
   --> crates\auto-lang\src\vm\codegen.rs:588:28
    |
588 |             Stmt::EnumDecl(enum_decl) => {
    |                            ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_enum_decl`

warning: unused variable: `spec_decl`
   --> crates\auto-lang\src\vm\codegen.rs:595:28
    |
595 |             Stmt::SpecDecl(spec_decl) => {
    |                            ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_spec_decl`

warning: unused variable: `loop_exit`
   --> crates\auto-lang\src\vm\codegen.rs:659:33
    |
659 | ...                   let loop_exit = self.code.len();
    |                           ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_loop_exit`

warning: unused variable: `call`
   --> crates\auto-lang\src\vm\codegen.rs:672:50
    |
672 |                         } else if let Expr::Call(call) = &for_stmt.range {
    |                                                  ^^^^ help: if this is intentional, prefix it with an underscore: `_call`

warning: unused variable: `loop_exit`
   --> crates\auto-lang\src\vm\codegen.rs:723:33
    |
723 | ...                   let loop_exit = self.code.len();
    |                           ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_loop_exit`

warning: unused variable: `loop_exit`
   --> crates\auto-lang\src\vm\codegen.rs:804:33
    |
804 | ...                   let loop_exit = self.code.len();
    |                           ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_loop_exit`

warning: unused variable: `loop_exit`
   --> crates\auto-lang\src\vm\codegen.rs:844:29
    |
844 |                         let loop_exit = self.code.len();
    |                             ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_loop_exit`

warning: unused variable: `loop_exit`
   --> crates\auto-lang\src\vm\codegen.rs:868:29
    |
868 |                         let loop_exit = self.code.len();
    |                             ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_loop_exit`

warning: unused variable: `loop_exit`
   --> crates\auto-lang\src\vm\codegen.rs:934:29
    |
934 |                         let loop_exit = self.code.len();
    |                             ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_loop_exit`

warning: variable does not need to be mutable
    --> crates\auto-lang\src\vm\codegen.rs:1906:21
     |
1906 |                 let mut func_name = match call.name.as_ref() {
     |                     ----^^^^^^^^^
     |                     |
     |                     help: remove this `mut`

warning: unused variable: `key`
    --> crates\auto-lang\src\vm\codegen.rs:1326:51
     |
1326 | ...                   crate::ast::Arg::Pair(key, expr) => {
     |                                             ^^^ help: if this is intentional, prefix it with an underscore: `_key`

warning: unused variable: `key`
    --> crates\auto-lang\src\vm\codegen.rs:1388:51
     |
1388 | ...                   crate::ast::Arg::Pair(key, expr) => {
     |                                             ^^^ help: if this is intentional, prefix it with an underscore: `_key`

warning: unused variable: `key`
    --> crates\auto-lang\src\vm\codegen.rs:1856:67
     |
1856 | ...                   crate::ast::Arg::Pair(key, expr) => {
     |                                             ^^^ help: if this is intentional, prefix it with an underscore: `_key`

warning: variable does not need to be mutable
    --> crates\auto-lang\src\vm\codegen.rs:2698:21
     |
2698 |                 let mut inner_exclude = exclude.clone();
     |                     ----^^^^^^^^^^^^^
     |                     |
     |                     help: remove this `mut`

warning: unused variable: `class_type`
    --> crates\auto-lang\src\vm\codegen.rs:3158:23
     |
3158 |             if let Ok(class_type) = self.generic_registry.get_or_create_type(&type_decl.name.to_string(), type_args) {
     |                       ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_class_type`

warning: unused variable: `range_value`
   --> crates\auto-lang\src\vm\engine.rs:580:25
    |
580 |                     let range_value = auto_val::Value::Range(start, end);
    |                         ^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_range_value`

warning: unused variable: `range_value`
   --> crates\auto-lang\src\vm\engine.rs:603:25
    |
603 |                     let range_value = auto_val::Value::RangeEq(start, end);
    |                         ^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_range_value`

warning: unused variable: `i`
    --> crates\auto-lang\src\vm\engine.rs:1525:25
     |
1525 |                     for i in 0..capture_count {
     |                         ^ help: if this is intentional, prefix it with an underscore: `_i`

warning: unused variable: `var_name`
    --> crates\auto-lang\src\vm\engine.rs:1563:25
     |
1563 |                     let var_name = if let Some(var_name_bytes) = strings.get(var_name_idx) {
     |                         ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_var_name`

warning: unused variable: `arg_count`
    --> crates\auto-lang\src\vm\engine.rs:1655:25
     |
1655 |                     let arg_count = self.flash.read_u8(task.ip) as usize;
     |                         ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_arg_count`

warning: unused variable: `val`
   --> crates\auto-lang\src\vm\generic_registry.rs:331:45
    |
331 |             (SpecializedPair::IntInt { key, val }, 0) => Some(Value::Int(*key)),
    |                                             ^^^ help: try ignoring the field: `val: _`

warning: unused variable: `val`
   --> crates\auto-lang\src\vm\generic_registry.rs:333:46
    |
333 |             (SpecializedPair::IntBool { key, val }, 0) => Some(Value::Int(*key)),
    |                                              ^^^ help: try ignoring the field: `val: _`

warning: unused variable: `val`
   --> crates\auto-lang\src\vm\generic_registry.rs:335:46
    |
335 |             (SpecializedPair::BoolInt { key, val }, 0) => Some(Value::Bool(*key)),
    |                                              ^^^ help: try ignoring the field: `val: _`

warning: unused variable: `val`
   --> crates\auto-lang\src\vm\generic_registry.rs:337:47
    |
337 |             (SpecializedPair::IntValue { key, val }, 0) => Some(Value::Int(*key)),
    |                                               ^^^ help: try ignoring the field: `val: _`

warning: unused variable: `val`
   --> crates\auto-lang\src\vm\generic_registry.rs:339:47
    |
339 |             (SpecializedPair::ValueInt { key, val }, 0) => Some(key.clone()),
    |                                               ^^^ help: try ignoring the field: `val: _`

warning: unused variable: `val`
   --> crates\auto-lang\src\vm\generic_registry.rs:341:48
    |
341 |             (SpecializedPair::BoolValue { key, val }, 0) => Some(Value::Bool(*key)),
    |                                                ^^^ help: try ignoring the field: `val: _`

warning: unused variable: `val`
   --> crates\auto-lang\src\vm\generic_registry.rs:343:48
    |
343 |             (SpecializedPair::ValueBool { key, val }, 0) => Some(key.clone()),
    |                                                ^^^ help: try ignoring the field: `val: _`

warning: unused variable: `val`
   --> crates\auto-lang\src\vm\generic_registry.rs:345:46
    |
345 |             (SpecializedPair::Generic { key, val }, 0) => Some(key.clone()),
    |                                              ^^^ help: try ignoring the field: `val: _`

warning: unused variable: `map`
   --> crates\auto-lang\src\vm\native.rs:852:21
    |
852 |         if let Some(map) = guard.as_any().downcast_ref::<AutoVMHashMap>() {
    |                     ^^^ help: if this is intentional, prefix it with an underscore: `_map`

warning: unused variable: `args`
   --> crates\auto-lang\src\lib.rs:464:40
    |
464 | pub fn eval_config_with_vm(code: &str, args: &Obj, univ: Universe) -> AutoResult<Value> {
    |                                        ^^^^ help: if this is intentional, prefix it with an underscore: `_args`

warning: unused variable: `univ`
   --> crates\auto-lang\src\lib.rs:464:52
    |
464 | pub fn eval_config_with_vm(code: &str, args: &Obj, univ: Universe) -> AutoResult<Value> {
    |                                                    ^^^^ help: if this is intentional, prefix it with an underscore: `_univ`

warning: type `codegen::TypeInfo` is more private than the item `Codegen::types`
  --> crates\auto-lang\src\vm\codegen.rs:92:5
   |
92 |     pub types: HashMap<String, TypeInfo>,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ field `Codegen::types` is reachable at visibility `pub`
   |
note: but type `codegen::TypeInfo` is only usable at visibility `pub(self)`
  --> crates\auto-lang\src\vm\codegen.rs:35:1
   |
35 | struct TypeInfo {
   | ^^^^^^^^^^^^^^^
   = note: `#[warn(private_interfaces)]` on by default

warning: type `ParamInfo` is more private than the item `Codegen::fn_params`
   --> crates\auto-lang\src\vm\codegen.rs:105:5
    |
105 |     pub fn_params: HashMap<String, Vec<ParamInfo>>,
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ field `Codegen::fn_params` is reachable at visibility `pub`
    |
note: but type `ParamInfo` is only usable at visibility `pub(self)`
   --> crates\auto-lang\src\vm\codegen.rs:43:1
    |
 43 | struct ParamInfo {
    | ^^^^^^^^^^^^^^^^

warning: type `codegen::TypeInfo` is more private than the item `Codegen::get_type`
    --> crates\auto-lang\src\vm\codegen.rs:3206:5
     |
3206 |     pub fn get_type(&self, name: &str) -> Option<&TypeInfo> {
     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ method `Codegen::get_type` is reachable at visibility `pub`
     |
note: but type `codegen::TypeInfo` is only usable at visibility `pub(self)`
    --> crates\auto-lang\src\vm\codegen.rs:35:1
     |
  35 | struct TypeInfo {
     | ^^^^^^^^^^^^^^^

warning: method `touch` is never used
  --> crates\auto-lang\src\query.rs:58:8
   |
47 | impl CacheEntry {
   | --------------- method in this implementation
...
58 |     fn touch(&mut self) {
   |        ^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: method `parse_u64` is never used
    --> crates\auto-lang\src\parser.rs:1788:8
     |
 893 | impl<'a> Parser<'a> {
     | ------------------- method in this implementation
...
1788 |     fn parse_u64(&mut self) -> AutoResult<Expr> {
     |        ^^^^^^^^^

warning: function `format_value` is never used
  --> crates\auto-lang\src\repl.rs:97:4
   |
97 | fn format_value(value: &Value, uni: &Shared<Universe>) -> String {
   |    ^^^^^^^^^^^^

warning: function `try_command` is never used
   --> crates\auto-lang\src\repl.rs:174:4
    |
174 | fn try_command(line: &str, interpreter: &mut interp::Interpreter) -> CmdResult {
    |    ^^^^^^^^^^^

warning: fields `current_fn` and `current_scope` are never read
  --> crates\auto-lang\src\trans\rust.rs:75:5
   |
63 | pub struct RustTrans {
   |            --------- fields in this struct
...
75 |     current_fn: Option<AutoStr>,
   |     ^^^^^^^^^^
76 |     current_scope: Option<crate::scope::Sid>,
   |     ^^^^^^^^^^^^^

warning: field `name` is never read
  --> crates\auto-lang\src\vm\codegen.rs:36:9
   |
35 | struct TypeInfo {
   |        -------- field in this struct
36 |     pub name: String,
   |         ^^^^
   |
   = note: `TypeInfo` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: methods `infer_expr_type`, `emit_store_ref`, `emit_store_mut_ref`, and `current_captured_vars_mut` are never used
    --> crates\auto-lang\src\vm\codegen.rs:2337:8
     |
 108 | impl Codegen {
     | ------------ methods in this implementation
...
2337 |     fn infer_expr_type(&self, expr: &Expr) -> Option<crate::ast::Type> {
     |        ^^^^^^^^^^^^^^^
...
2504 |     fn emit_store_ref(&mut self, index: usize) {
     |        ^^^^^^^^^^^^^^
...
2519 |     fn emit_store_mut_ref(&mut self, index: usize) {
     |        ^^^^^^^^^^^^^^^^^^
...
2634 |     fn current_captured_vars_mut(&mut self) -> &mut HashMap<String, usize> {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: associated functions `pop_value_as_int`, `pop_value_as_float`, `pop_value_as_double`, and `pop_value_as_string_index` are never used
   --> crates\auto-lang\src\vm\engine.rs:259:8
    |
111 | impl AutoVM {
    | ----------- associated functions in this implementation
...
259 |     fn pop_value_as_int(ram: &mut VirtualRAM) -> i32 {
    |        ^^^^^^^^^^^^^^^^
...
264 |     fn pop_value_as_float(ram: &mut VirtualRAM) -> f32 {
    |        ^^^^^^^^^^^^^^^^^^
...
269 |     fn pop_value_as_double(ram: &mut VirtualRAM) -> f64 {
    |        ^^^^^^^^^^^^^^^^^^^
...
274 |     fn pop_value_as_string_index(ram: &mut VirtualRAM) -> i32 {
    |        ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: creating a mutable reference to mutable static
   --> crates\auto-lang\src\vm\list_data.rs:184:26
    |
184 |                 unsafe { &mut DUMMY }
    |                          ^^^^^^^^^^ mutable reference to mutable static
    |
    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/static-mut-references.html>
    = note: mutable references to mutable statics are dangerous; it's undefined behavior if any other pointer to the static is used or if any other reference is created for the static while the mutable reference lives
    = note: `#[warn(static_mut_refs)]` (part of `#[warn(rust_2024_compatibility)]`) on by default
help: use `&raw mut` instead to create a raw pointer
    |
184 |                 unsafe { &raw mut DUMMY }
    |                           +++

warning: unnecessary transmute
  --> crates\auto-lang\src\vm\virt_memory.rs:26:28
   |
26 |         Self { i: unsafe { std::mem::transmute(val) } }
   |                            ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unnecessary_transmutes)]` on by default
help: replace this with
   |
26 -         Self { i: unsafe { std::mem::transmute(val) } }
26 +         Self { i: unsafe { f32::to_bits(val).cast_signed() } }
   |

warning: unnecessary transmute
   --> crates\auto-lang\src\vm\virt_memory.rs:209:34
    |
209 |         let bits: i32 = unsafe { std::mem::transmute(val) };
    |                                  ^^^^^^^^^^^^^^^^^^^^^^^^
    |
help: replace this with
    |
209 -         let bits: i32 = unsafe { std::mem::transmute(val) };
209 +         let bits: i32 = unsafe { f32::to_bits(val).cast_signed() };
    |

warning: unnecessary transmute
   --> crates\auto-lang\src\vm\virt_memory.rs:216:18
    |
216 |         unsafe { std::mem::transmute(bits) }
    |                  ^^^^^^^^^^^^^^^^^^^^^^^^^
    |
help: replace this with
    |
216 -         unsafe { std::mem::transmute(bits) }
216 +         unsafe { f32::from_bits(i32::cast_unsigned(bits)) }
    |

warning: unnecessary transmute
   --> crates\auto-lang\src\vm\virt_memory.rs:224:34
    |
224 |         let bits: u64 = unsafe { std::mem::transmute(val) };
    |                                  -------------------^^^^^
    |                                  |
    |                                  help: replace this with: `f64::to_bits`

warning: unnecessary transmute
   --> crates\auto-lang\src\vm\virt_memory.rs:236:18
    |
236 |         unsafe { std::mem::transmute(bits) }
    |                  -------------------^^^^^^
    |                  |
    |                  help: replace this with: `f64::from_bits`

warning: `auto-lang` (lib) generated 606 warnings (run `cargo fix --lib -p auto-lang` to apply 61 suggestions)
warning: use of deprecated struct `auto_lang::interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
 --> crates\auto-gen\src\data.rs:3:24
  |
3 | use auto_lang::interp::Interpreter;
  |                        ^^^^^^^^^^^
  |
  = note: `#[warn(deprecated)]` on by default

warning: use of deprecated struct `auto_lang::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
 --> crates\auto-gen\src\data.rs:4:16
  |
4 | use auto_lang::Universe;
  |                ^^^^^^^^

warning: use of deprecated struct `auto_lang::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-gen\src\data.rs:18:23
   |
18 |     pub scope: Shared<Universe>,
   |                       ^^^^^^^^

warning: use of deprecated struct `auto_lang::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-gen\src\data.rs:58:36
   |
58 |                 let mut universe = Universe::new();
   |                                    ^^^^^^^^

warning: use of deprecated struct `auto_lang::interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
  --> crates\auto-gen\src\data.rs:80:25
   |
80 |         let mut inter = Interpreter::new();
   |                         ^^^^^^^^^^^

warning: use of deprecated struct `auto_lang::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-gen\src\data.rs:106:28
    |
106 |         let mut universe = Universe::new();
    |                            ^^^^^^^^

warning: use of deprecated struct `auto_lang::interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-gen\src\generator.rs:177:32
    |
177 |         use auto_lang::interp::Interpreter;
    |                                ^^^^^^^^^^^

warning: use of deprecated struct `auto_lang::interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-gen\src\generator.rs:186:25
    |
186 |         let mut inter = Interpreter::with_univ(loaded_data.scope.clone());
    |                         ^^^^^^^^^^^

warning: use of deprecated struct `auto_lang::interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
 --> crates\auto-gen\src\template.rs:4:24
  |
4 | use auto_lang::interp::Interpreter;
  |                        ^^^^^^^^^^^

warning: use of deprecated struct `auto_lang::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
 --> crates\auto-gen\src\template.rs:5:16
  |
5 | use auto_lang::Universe;
  |                ^^^^^^^^

warning: use of deprecated struct `auto_lang::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-gen\src\template.rs:94:39
   |
94 |         let mut universe = auto_lang::Universe::new();
   |                                       ^^^^^^^^

warning: use of deprecated struct `auto_lang::interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
  --> crates\auto-gen\src\template.rs:98:32
   |
98 |             auto_lang::interp::Interpreter::with_scope(universe).with_fstr_note(self.fstr_note);
   |                                ^^^^^^^^^^^

warning: use of deprecated struct `auto_lang::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-gen\src\template.rs:120:27
    |
120 |         universe: &Shared<Universe>,
    |                           ^^^^^^^^

warning: use of deprecated struct `auto_lang::interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-gen\src\template.rs:126:25
    |
126 |         let mut inter = Interpreter::with_univ(universe.clone()).with_fstr_note(self.fstr_note);
    |                         ^^^^^^^^^^^

warning: use of deprecated struct `auto_lang::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-gen\src\template.rs:170:77
    |
170 |     fn preprocess_use_statements(&self, source: &AutoStr, universe: &Shared<Universe>) -> GenResult<AutoStr> {
    |                                                                             ^^^^^^^^

warning: use of deprecated struct `auto_lang::interp::Interpreter`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-gen\src\template.rs:187:29
    |
187 |             let mut inter = Interpreter::with_univ(universe.clone()).with_fstr_note(self.fstr_note);
    |                             ^^^^^^^^^^^

warning: use of deprecated field `auto_lang::interp::Interpreter::scope`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto-gen\src\data.rs:100:28
    |
100 |                     scope: inter.scope.clone(),
    |                            ^^^^^^^^^^^

warning: `auto-gen` (lib) generated 17 warnings
warning: unused import: `crate::version::Version`
 --> crates\auto-man\src\lock.rs:7:5
  |
7 | use crate::version::Version;
  |     ^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `std::io::Write`
  --> crates\auto-man\src\lock.rs:11:5
   |
11 | use std::io::Write;
   |     ^^^^^^^^^^^^^^

warning: unused import: `std::path::PathBuf`
  --> crates\auto-man\src\target.rs:16:5
   |
16 | use std::path::PathBuf;
   |     ^^^^^^^^^^^^^^^^^^

warning: unused imports: `ArtifactMetadata`, `CacheStatistics`, `CompilationTarget`, and `IntegrityReport`
  --> crates\auto-man\src\automan.rs:19:46
   |
19 | use auto_cache::{AutoManCache, ArtifactType, ArtifactMetadata, CacheStatistics, CompilationTarget, IntegrityReport};
   |                                              ^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^

warning: unused import: `pull::*`
  --> crates\auto-man\src\lib.rs:50:9
   |
50 | pub use pull::*;
   |         ^^^^^^^

warning: use of deprecated struct `auto_lang::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
 --> crates\auto-man\src\asset.rs:5:16
  |
5 | use auto_lang::Universe;
  |                ^^^^^^^^
  |
  = note: `#[warn(deprecated)]` on by default

warning: use of deprecated struct `auto_lang::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-man\src\asset.rs:43:36
   |
43 |                 let mut universe = Universe::new();
   |                                    ^^^^^^^^

warning: use of deprecated struct `auto_gen::AutoGen`: Use CodeGenerator instead
  --> crates\auto-man\src\builder\iar.rs:11:14
   |
11 |     pub gen: AutoGen,
   |              ^^^^^^^

warning: use of deprecated struct `auto_gen::AutoGen`: Use CodeGenerator instead
  --> crates\auto-man\src\builder\iar.rs:12:23
   |
12 |     pub app_gens: Vec<AutoGen>,
   |                       ^^^^^^^

warning: use of deprecated struct `auto_gen::AutoGen`: Use CodeGenerator instead
  --> crates\auto-man\src\builder\iar.rs:26:36
   |
26 |     fn new_gen(path: &AutoPath) -> AutoGen {
   |                                    ^^^^^^^

warning: use of deprecated struct `auto_gen::AutoGen`: Use CodeGenerator instead
  --> crates\auto-man\src\builder\iar.rs:27:9
   |
27 |         AutoGen::new().out(path.clone()).note('@').rename(true)
   |         ^^^^^^^

warning: use of deprecated struct `auto_gen::AutoGen`: Use CodeGenerator instead
  --> crates\auto-man\src\builder\ghs.rs:12:14
   |
12 |     pub gen: AutoGen,
   |              ^^^^^^^

warning: use of deprecated struct `auto_gen::OneGen`: Use CodeGenerator instead
  --> crates\auto-man\src\builder\ghs.rs:13:19
   |
13 |     pub apps: Vec<OneGen>,
   |                   ^^^^^^

warning: use of deprecated struct `auto_gen::OneGen`: Use CodeGenerator instead
  --> crates\auto-man\src\builder\ghs.rs:14:26
   |
14 |     pub subprojects: Vec<OneGen>,
   |                          ^^^^^^

warning: use of deprecated struct `auto_gen::AutoGen`: Use CodeGenerator instead
  --> crates\auto-man\src\builder\ghs.rs:20:19
   |
20 |         let gen = AutoGen::new().out(path.clone()).note('$');
   |                   ^^^^^^^

warning: use of deprecated struct `auto_gen::OneGen`: Use CodeGenerator instead
  --> crates\auto-man\src\builder\ghs.rs:67:23
   |
67 |         let lib_gen = OneGen::new(lib_mold, atom).out(AutoPath::new(ghs_loc.clone()));
   |                       ^^^^^^

warning: use of deprecated struct `auto_gen::OneGen`: Use CodeGenerator instead
   --> crates\auto-man\src\builder\ghs.rs:165:27
    |
165 |             let app_gen = OneGen::new(app_mold, atom).out(AutoPath::new(ghs_loc.clone()));
    |                           ^^^^^^

warning: use of deprecated struct `auto_gen::OneGen`: Use CodeGenerator instead
   --> crates\auto-man\src\builder\ghs.rs:196:31
    |
196 |                 let lib_gen = OneGen::new(lib_mold, atom).out(AutoPath::new(ghs_loc.clone()));
    |                               ^^^^^^

warning: use of deprecated struct `auto_gen::AutoGen`: Use CodeGenerator instead
 --> crates\auto-man\src\builder\ninja\builder.rs:2:15
  |
2 | use auto_gen::AutoGen;
  |               ^^^^^^^

warning: use of deprecated struct `auto_gen::AutoGen`: Use CodeGenerator instead
  --> crates\auto-man\src\builder\ninja\builder.rs:42:14
   |
42 |     pub gen: AutoGen,
   |              ^^^^^^^

warning: use of deprecated struct `auto_gen::AutoGen`: Use CodeGenerator instead
  --> crates\auto-man\src\builder\ninja\builder.rs:63:36
   |
63 |     fn new_gen(path: &AutoPath) -> AutoGen {
   |                                    ^^^^^^^

warning: use of deprecated struct `auto_gen::AutoGen`: Use CodeGenerator instead
  --> crates\auto-man\src\builder\ninja\builder.rs:64:9
   |
64 |         AutoGen::new().out(path.clone()).note('$').rename(true)
   |         ^^^^^^^

warning: use of deprecated struct `auto_lang::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto-man\src\pac.rs:16:16
   |
16 | use auto_lang::Universe;
   |                ^^^^^^^^

warning: use of deprecated struct `auto_lang::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-man\src\pac.rs:771:25
    |
771 |                         Universe::default(),
    |                         ^^^^^^^^

warning: use of deprecated struct `auto_lang::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
 --> crates\auto-man\src\automan.rs:8:16
  |
8 | use auto_lang::Universe;
  |                ^^^^^^^^

warning: use of deprecated struct `auto_lang::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-man\src\automan.rs:140:23
    |
140 |         let mut env = Universe::new();
    |                       ^^^^^^^^

warning: use of deprecated field `auto_lang::interp::Interpreter::result`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
  --> crates\auto-man\src\asset.rs:55:30
   |
55 |                 let result = auto_code.unwrap().result;
   |                              ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `auto_gen::AutoGen::out`: Use CodeGenerator instead
  --> crates\auto-man\src\builder\iar.rs:72:27
   |
72 |         let project_dir = self.gen.out.path();
   |                           ^^^^^^^^^^^^

warning: use of deprecated field `auto_gen::AutoGen::out`: Use CodeGenerator instead
  --> crates\auto-man\src\builder\ghs.rs:89:27
   |
89 |         let project_dir = self.gen.out.path();
   |                           ^^^^^^^^^^^^

warning: use of deprecated field `auto_lang::Universe::code_paks`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-man\src\target.rs:607:32
    |
607 |             for (_sid, pak) in uni.borrow().code_paks.iter() {
    |                                ^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `auto_lang::Universe::code_paks`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-man\src\target.rs:750:40
    |
750 |                     for (_sid, pak) in uni.borrow().code_paks.iter() {
    |                                        ^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated field `auto_lang::Universe::code_paks`: Use Database + ExecutionEngine instead (see Plan 064)
   --> crates\auto-man\src\target.rs:783:40
    |
783 |                     for (_sid, pak) in uni.borrow().code_paks.iter() {
    |                                        ^^^^^^^^^^^^^^^^^^^^^^

warning: `auto-man` (lib) generated 32 warnings (run `cargo fix --lib -p auto-man` to apply 5 suggestions)
warning: use of deprecated struct `auto_lang::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
 --> crates\auto\src\cmd_a2c_stdlib.rs:4:5
  |
4 |     Universe,
  |     ^^^^^^^^
  |
  = note: `#[warn(deprecated)]` on by default

warning: use of deprecated struct `auto_lang::Universe`: Use Database + ExecutionEngine instead (see Plan 064)
  --> crates\auto\src\cmd_a2c_stdlib.rs:63:42
   |
63 |         let scope = Rc::new(RefCell::new(Universe::new()));
   |                                          ^^^^^^^^

warning: use of deprecated method `auto_lang::trans::c::CTrans::set_scope`: Use with_database() instead (Phase 066)
  --> crates\auto\src\cmd_a2c_stdlib.rs:75:15
   |
75 |         trans.set_scope(parser.scope.clone());
   |               ^^^^^^^^^

warning: use of deprecated field `auto_lang::interp::Interpreter::result`: Use run() or run_bigvm() instead (Plan 068 Phase 9).
   --> crates\auto\src\main.rs:312:28
    |
312 |             println!("{}", c.result.repr());
    |                            ^^^^^^^^

warning: `auto` (bin "auto") generated 4 warnings
    Finished `release` profile [optimized] target(s) in 0.74s
     Running `target\release\auto.exe run tmp/test_method_simple.at 2`
error: unexpected argument '2' found

Usage: auto.exe run [OPTIONS] <PATH>

For more information, try '--help'.
error: process didn't exit successfully: `target\release\auto.exe run tmp/test_method_simple.at 2` (exit code: 2)
