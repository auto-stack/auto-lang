use crate::eval::{Evaler, EvalMode};
use crate::scope::Universe;
use autoval::Value;
use std::rc::Rc;
use std::cell::RefCell;
use crate::parser;

pub struct Interpreter {
    pub evaler: Evaler,
    pub scope: Rc<RefCell<Universe>>,
    pub result: Value,
}

impl Interpreter {
    pub fn new() -> Self {
        let scope = Rc::new(RefCell::new(Universe::new()));
        let interpreter = Self { 
            evaler: Evaler::new(scope.clone()),
            scope, 
            result: Value::Nil 
        };
        interpreter
    }

    pub fn with_scope(scope: Universe) -> Self {
        let scope = Rc::new(RefCell::new(scope));
        let interpreter = Self { 
            evaler: Evaler::new(scope.clone()),
            scope, 
            result: Value::Nil 
        };
        interpreter
    }
    
    pub fn wit_eval_mode(mut self, mode: EvalMode) -> Self {
        self.evaler = self.evaler.with_mode(mode);
        self
    }

    pub fn interpret(&mut self, code: &str) -> Result<(), String> {
        let ast = parser::parse(code, self.scope.clone())?;
        let result = self.evaler.eval(&ast);
        self.result = result;
        Ok(())
    }

    pub fn eval(&mut self, code: &str) -> Value {
        match self.interpret(code) {
            Ok(_) => self.result.clone(),
            Err(e) => Value::Error(e),
        }
    }
}
