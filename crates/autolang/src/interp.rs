use crate::eval::Evaler;
use crate::scope::Universe;
use autoval::value::Value;
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

    pub fn interpret(&mut self, code: &str) -> Result<(), String> {
        let ast = parser::parse(code, &mut *self.scope.borrow_mut())?;
        let result = self.evaler.eval(&ast);
        self.result = result;
        Ok(())
    }
}
