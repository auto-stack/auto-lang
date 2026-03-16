//! AutoMath Parser - Function-Style Math to Target Notation
//!
//! Converts function-style math expressions to various target formats.

use super::ast::AdocMath;
use super::error::AdocResult;

/// AutoMath parser
pub struct AutoMathParser;

impl AutoMathParser {
    /// Parse function-style math into structured form
    pub fn parse(content: &str) -> AdocResult<AdocMath> {
        // For now, just return the content as-is
        // A full implementation would parse function syntax
        Ok(AdocMath::inline(content))
    }

    /// Convert math content to LaTeX notation
    pub fn to_latex(math: &AdocMath) -> String {
        let mut result = math.content.clone();

        // Convert function-style to LaTeX
        result = Self::convert_sum(&result);
        result = Self::convert_prod(&result);
        result = Self::convert_integral(&result);
        result = Self::convert_sqrt(&result);

        result
    }

    /// Convert math content to Typst notation
    pub fn to_typst(math: &AdocMath) -> String {
        let mut result = math.content.clone();

        // Typst uses similar syntax but with different delimiters
        result = Self::convert_sum_typst(&result);
        result = Self::convert_prod_typst(&result);

        result
    }

    /// Convert math content to MathML
    pub fn to_mathml(math: &AdocMath) -> String {
        // Basic MathML conversion
        let content = &math.content;

        // Wrap in MathML tags
        if math.display {
            format!(
                "<math display=\"block\"><mrow><mi>{}</mi></mrow></math>",
                content
            )
        } else {
            format!("<math><mrow><mi>{}</mi></mrow></math>", content)
        }
    }

    // LaTeX conversion helpers

    fn convert_sum(content: &str) -> String {
        // sum(i=0..n, f(i)) → \sum_{i=0}^{n} f(i)
        Self::replace_math_function(content, "sum", "\\sum")
    }

    fn convert_prod(content: &str) -> String {
        // prod(i=0..n, f(i)) → \prod_{i=0}^{n} f(i)
        Self::replace_math_function(content, "prod", "\\prod")
    }

    fn convert_integral(content: &str) -> String {
        // integral(a, b, f(x)) → \int_{a}^{b} f(x)
        content.replace("integral(", "\\int ")
    }

    fn convert_sqrt(content: &str) -> String {
        // sqrt(x) → \sqrt{x}
        content.replace("sqrt(", "\\sqrt{")
    }

    fn replace_math_function(content: &str, func: &str, latex: &str) -> String {
        let pattern = format!("{}(", func);
        let mut result = String::new();
        let chars: Vec<char> = content.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            // Check for function pattern
            if i + pattern.len() <= chars.len() {
                let slice: String = chars[i..i + pattern.len()].iter().collect();
                if slice == pattern {
                    i += pattern.len();

                    // Parse arguments
                    let mut paren_level = 1;
                    let mut args = String::new();

                    while i < chars.len() && paren_level > 0 {
                        if chars[i] == '(' {
                            paren_level += 1;
                        } else if chars[i] == ')' {
                            paren_level -= 1;
                            if paren_level == 0 {
                                i += 1;
                                break;
                            }
                        }
                        args.push(chars[i]);
                        i += 1;
                    }

                    // Parse i=a..b, expr
                    if let Some(dot_pos) = args.find("..") {
                        let before_dot = &args[..dot_pos];
                        let after_dot = &args[dot_pos + 2..];

                        if let Some(comma_pos) = after_dot.find(',') {
                            let end_val = &after_dot[..comma_pos].trim();
                            let expr = &after_dot[comma_pos + 1..].trim();

                            // Extract variable and start value
                            if let Some(eq_pos) = before_dot.find('=') {
                                let var = &before_dot[..eq_pos].trim();
                                let start_val = &before_dot[eq_pos + 1..].trim();

                                result.push_str(&format!(
                                    "{}_{{{}={}}}^{{{}}} {}",
                                    latex, var, start_val, end_val, expr
                                ));
                            }
                        }
                    }
                    continue;
                }
            }

            result.push(chars[i]);
            i += 1;
        }

        result
    }

    // Typst conversion helpers

    fn convert_sum_typst(content: &str) -> String {
        // sum(i=0..n, f(i)) → sum_i^n f(i)
        Self::replace_math_function_typst(content, "sum")
    }

    fn convert_prod_typst(content: &str) -> String {
        // prod(i=0..n, f(i)) → product_i^n f(i)
        Self::replace_math_function_typst(content, "prod")
    }

    fn replace_math_function_typst(content: &str, func: &str) -> String {
        let pattern = format!("{}(", func);
        let mut result = String::new();
        let chars: Vec<char> = content.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if i + pattern.len() <= chars.len() {
                let slice: String = chars[i..i + pattern.len()].iter().collect();
                if slice == pattern {
                    i += pattern.len();

                    let mut paren_level = 1;
                    let mut args = String::new();

                    while i < chars.len() && paren_level > 0 {
                        if chars[i] == '(' {
                            paren_level += 1;
                        } else if chars[i] == ')' {
                            paren_level -= 1;
                            if paren_level == 0 {
                                i += 1;
                                break;
                            }
                        }
                        args.push(chars[i]);
                        i += 1;
                    }

                    // Parse i=a..b, expr
                    if let Some(dot_pos) = args.find("..") {
                        let before_dot = &args[..dot_pos];
                        let after_dot = &args[dot_pos + 2..];

                        if let Some(comma_pos) = after_dot.find(',') {
                            let end_val = &after_dot[..comma_pos].trim();
                            let expr = &after_dot[comma_pos + 1..].trim();

                            if let Some(eq_pos) = before_dot.find('=') {
                                let var = &before_dot[..eq_pos].trim();
                                result.push_str(&format!("{}_{}^{} {}", func, var, end_val, expr));
                            }
                        }
                    }
                    continue;
                }
            }

            result.push(chars[i]);
            i += 1;
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latex_sum() {
        let math = AdocMath::inline("sum(i=0..n, i^2)");
        let latex = AutoMathParser::to_latex(&math);

        assert!(latex.contains("\\sum"));
    }

    #[test]
    fn test_typst_sum() {
        let math = AdocMath::inline("sum(i=0..n, i^2)");
        let typst = AutoMathParser::to_typst(&math);

        assert!(typst.contains("sum_"));
    }
}
