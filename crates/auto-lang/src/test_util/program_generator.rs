// Plan 266 Phase 4: Differential Testing — Program Generator
//
// Generates type-correct random Auto programs for differential testing.
// Both AutoVM and a2r execute the same program; output must match.

/// Supported types for program generation
#[derive(Debug, Clone, PartialEq)]
enum GenType {
    Int,
    Str,
    Bool,
    F64,
}

impl std::fmt::Display for GenType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GenType::Int => write!(f, "int"),
            GenType::Str => write!(f, "str"),
            GenType::Bool => write!(f, "bool"),
            GenType::F64 => write!(f, "f64"),
        }
    }
}

/// A generated function signature
#[derive(Debug, Clone)]
struct GenFunc {
    name: String,
    params: Vec<(String, GenType)>,
    ret_type: GenType,
}

pub struct ProgramGenerator {
    rng: fastrand::Rng,
    functions: Vec<GenFunc>,
    depth_limit: u32,
    var_counter: u32,
    /// Seed for reproducibility
    seed: u64,
}

impl ProgramGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: fastrand::Rng::with_seed(seed),
            functions: Vec::new(),
            depth_limit: 3,
            var_counter: 0,
            seed,
        }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    fn fresh_var(&mut self) -> String {
        let id = self.var_counter;
        self.var_counter += 1;
        format!("v{}", id)
    }

    /// Generate a complete Auto program with functions and a main that prints results.
    pub fn generate_program(&mut self) -> String {
        self.functions.clear();
        self.var_counter = 0;

        let mut code = String::new();

        // Generate 1-3 helper functions
        let func_count = self.rng.usize(1..=3);
        for _ in 0..func_count {
            let func = self.gen_func_signature();
            let body = self.gen_func_body(&func);
            let params_str = func.params.iter()
                .map(|(n, t)| format!("{} {}", n, t))
                .collect::<Vec<_>>()
                .join(", ");
            code.push_str(&format!("fn {}({}) {} {{\n    {}\n}}\n\n",
                func.name, params_str, func.ret_type, body));
            self.functions.push(func);
        }

        // Generate main with print statements
        code.push_str("fn main() {\n");
        let stmt_count = self.rng.usize(2..=5);
        for _ in 0..stmt_count {
            let expr = self.gen_expr(&GenType::Int, self.depth_limit);
            code.push_str(&format!("    print({})\n", expr));
        }
        code.push_str("}\n");
        code
    }

    fn gen_func_signature(&mut self) -> GenFunc {
        let ret_types = [GenType::Int, GenType::Str, GenType::Bool];
        let ret_type = ret_types[self.rng.usize(0..ret_types.len())].clone();

        let param_count = self.rng.usize(1..=3);
        let mut params = Vec::new();
        for _ in 0..param_count {
            let name = self.fresh_var();
            let ptype = match self.rng.usize(0..4) {
                0 => GenType::Int,
                1 => GenType::Str,
                2 => GenType::Bool,
                _ => GenType::F64,
            };
            params.push((name, ptype));
        }

        let func_id = self.functions.len();
        GenFunc {
            name: format!("f{}", func_id),
            params,
            ret_type,
        }
    }

    fn gen_func_body(&mut self, func: &GenFunc) -> String {
        // Generate a return expression matching the return type
        let expr = self.gen_expr_in_func(func, &func.ret_type, 2);
        format!("return {}", expr)
    }

    fn gen_expr(&mut self, target: &GenType, depth: u32) -> String {
        if depth == 0 {
            return self.gen_literal(target);
        }

        match target {
            GenType::Int => {
                match self.rng.usize(0..5) {
                    0 => format!("{} + {}", self.gen_expr(target, depth - 1), self.gen_expr(target, depth - 1)),
                    1 => format!("{} * {}", self.gen_expr(target, depth - 1), self.gen_expr(target, depth - 1)),
                    2 => format!("{} - {}", self.gen_expr(target, depth - 1), self.gen_literal(target)),
                    3 => {
                        let divisor = self.rng.i32(1..=10);
                        format!("{} / {}", self.gen_expr(target, depth - 1), divisor)
                    }
                    _ => self.gen_literal(target),
                }
            }
            GenType::Str => {
                match self.rng.usize(0..3) {
                    0 => format!("{} + {}", self.gen_expr(target, depth - 1), self.gen_expr(target, depth - 1)),
                    1 => {
                        let var = self.fresh_var();
                        format!(r#"f"${}"#, var)
                    }
                    _ => self.gen_literal(target),
                }
            }
            GenType::Bool => {
                match self.rng.usize(0..3) {
                    0 => format!("{} < {}", self.gen_expr(&GenType::Int, depth - 1), self.gen_expr(&GenType::Int, depth - 1)),
                    1 => format!("{} == {}", self.gen_expr(&GenType::Int, depth - 1), self.gen_expr(&GenType::Int, depth - 1)),
                    _ => self.gen_literal(target),
                }
            }
            GenType::F64 => {
                match self.rng.usize(0..3) {
                    0 => format!("{} + {}", self.gen_expr(target, depth - 1), self.gen_expr(target, depth - 1)),
                    1 => format!("{} * {}", self.gen_literal(target), self.gen_expr(target, depth - 1)),
                    _ => self.gen_literal(target),
                }
            }
        }
    }

    fn gen_expr_in_func(&mut self, func: &GenFunc, target: &GenType, depth: u32) -> String {
        if depth == 0 {
            // Try to use a parameter if type matches
            for (name, ptype) in &func.params {
                if ptype == target {
                    return name.clone();
                }
            }
            return self.gen_literal(target);
        }

        // 20% chance to call the function recursively or use a param
        if !func.params.is_empty() && self.rng.usize(0..5) == 0 {
            let (pname, _) = &func.params[self.rng.usize(0..func.params.len())];
            return pname.clone();
        }

        self.gen_expr(target, depth)
    }

    fn gen_literal(&mut self, t: &GenType) -> String {
        match t {
            GenType::Int => self.rng.i32(-100..100).to_string(),
            GenType::Str => {
                let strs = ["\"a\"", "\"b\"", "\"hello\"", "\"x\"", "\"test\"", "\"\""];
                strs[self.rng.usize(0..strs.len())].to_string()
            }
            GenType::Bool => {
                if self.rng.bool() { "true".into() } else { "false".into() }
            }
            GenType::F64 => {
                let v = self.rng.f64() * 100.0;
                format!("{:.1}", v)
            }
        }
    }
}

/// Minimize a failing test case by removing statements one at a time.
pub fn minimize_program(source: &str, is_still_failing: impl Fn(&str) -> bool) -> String {
    let mut minimal = source.to_string();
    let mut changed = true;

    while changed {
        changed = false;
        let lines: Vec<&str> = minimal.lines().collect();
        for i in (0..lines.len()).rev() {
            let candidate: String = lines.iter().enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, l)| *l)
                .collect::<Vec<_>>()
                .join("\n");
            if is_still_failing(&candidate) {
                minimal = candidate;
                changed = true;
                break;
            }
        }
    }

    minimal
}

#[cfg(test)]
mod gen_tests {
    use super::*;

    #[test]
    fn test_generator_produces_valid_syntax() {
        for seed in 0..20 {
            let mut gen = ProgramGenerator::new(seed);
            let program = gen.generate_program();
            // Should contain fn main
            assert!(program.contains("fn main()"), "Seed {} missing main: {}", seed, program);
            // Should contain print
            assert!(program.contains("print("), "Seed {} missing print: {}", seed, program);
        }
    }

    #[test]
    fn test_generator_seed_reproducibility() {
        let mut gen1 = ProgramGenerator::new(42);
        let mut gen2 = ProgramGenerator::new(42);
        let p1 = gen1.generate_program();
        let p2 = gen2.generate_program();
        assert_eq!(p1, p2, "Same seed should produce same program");
    }

    #[test]
    fn test_minimize_removes_lines() {
        let source = "fn main() {\n    print(1)\n    print(2)\n    print(3)\n}\n";
        let result = minimize_program(source, |s| s.contains("print(3)"));
        assert!(result.lines().count() < source.lines().count());
    }
}
