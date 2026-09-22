//! # autodown_math — VM 轨 LaTeX→Unicode 符号转换（PLAN-084 T-01b）
//!
//! 数学"真渲染"的 bounded 实现：web 轨 katex 语义的 VM 等价子集。
//! 覆盖：希腊字母、常用运算/关系/箭头、上下标（单字符与 {..} 包裹）、
//! 常见符号命令。未命中命令剥反斜杠按原文保留（如 `\det` → `det`）；
//! 矩阵环境（\begin{pmatrix}…）不做结构化渲染——降级面板语义不变。
//!
//! 消费点（autodown_render）：math_inline span 内容、MathBlock 降级面板
//! 正文。只读纯函数，无依赖。

/// LaTeX 命令 → (Unicode 符号, 是否字母类)。字母类（希腊/变体字母）在
/// TeX 语义下吞命令后单个空格（`\lambda v` → `λv`）；算符/关系/箭头保留
/// 空格（`a \times b` → `a × b`）。未命中返回 None → 调用方剥反斜杠。
fn cmd_to_unicode(cmd: &str) -> Option<(&'static str, bool)> {
    // 字母类（吞空格）：
    if cmd.starts_with("alpha") || cmd.starts_with("beta") || cmd.starts_with("gamma")
        || cmd.starts_with("delta") || cmd.starts_with("epsilon") || cmd == "zeta"
        || cmd == "eta" || cmd.starts_with("theta") || cmd == "iota"
        || cmd.starts_with("kappa") || cmd.starts_with("lambda") || cmd == "mu"
        || cmd == "nu" || cmd == "xi" || cmd == "pi" || cmd.starts_with("rho")
        || cmd.starts_with("sigma") || cmd == "tau" || cmd.starts_with("upsilon")
        || cmd == "phi" || cmd == "varphi" || cmd == "chi" || cmd.starts_with("psi")
        || cmd.starts_with("omega") || cmd == "ell" || cmd == "hbar"
        || cmd.starts_with("Gamma") || cmd.starts_with("Delta") || cmd.starts_with("Theta")
        || cmd.starts_with("Lambda") || cmd == "Xi" || cmd == "Pi" || cmd.starts_with("Sigma")
        || cmd == "Phi" || cmd == "Psi" || cmd.starts_with("Omega")
    {
        let sym = match cmd {
            "alpha" => "α", "beta" => "β", "gamma" => "γ", "delta" => "δ",
            "epsilon" | "varepsilon" => "ε", "zeta" => "ζ", "eta" => "η",
            "theta" => "θ", "iota" => "ι", "kappa" => "κ", "lambda" => "λ",
            "mu" => "μ", "nu" => "ν", "xi" => "ξ", "pi" => "π", "rho" => "ρ",
            "sigma" => "σ", "tau" => "τ", "upsilon" => "υ", "phi" | "varphi" => "φ",
            "chi" => "χ", "psi" => "ψ", "omega" => "ω",
            "Gamma" => "Γ", "Delta" => "Δ", "Theta" => "Θ", "Lambda" => "Λ",
            "Xi" => "Ξ", "Pi" => "Π", "Sigma" => "Σ", "Phi" => "Φ",
            "Psi" => "Ψ", "Omega" => "Ω",
            "ell" => "ℓ", "hbar" => "ℏ",
            _ => return None,
        };
        return Some((sym, true));
    }
    let sym = match cmd {
        "times" => "×", "div" => "÷", "cdot" => "·", "pm" => "±",
        "mp" => "∓", "leq" | "le" => "≤", "geq" | "ge" => "≥",
        "neq" | "ne" => "≠", "approx" => "≈", "equiv" => "≡",
        "sim" => "∼", "propto" => "∝", "infty" => "∞", "sum" => "Σ",
        "prod" => "∏", "int" => "∫", "sqrt" => "√", "partial" => "∂",
        "nabla" => "∇",
        "rightarrow" | "to" => "→", "leftarrow" | "gets" => "←",
        "Rightarrow" => "⇒", "Leftarrow" => "⇐",
        "leftrightarrow" => "↔", "Leftrightarrow" => "⇔", "mapsto" => "↦",
        "in" => "∈", "notin" => "∉", "subset" => "⊂", "subseteq" => "⊆",
        "supset" => "⊃", "cup" => "∪", "cap" => "∩", "emptyset" => "∅",
        "forall" => "∀", "exists" => "∃", "neg" => "¬", "land" => "∧",
        "lor" => "∨", "langle" => "⟨", "rangle" => "⟩", "circ" => "∘",
        "bullet" => "•", "star" => "⋆", "oplus" => "⊕", "otimes" => "⊗",
        "perp" => "⊥", "parallel" => "∥", "angle" => "∠",
        "triangle" => "△", "degree" => "°", "prime" => "′",
        _ => return None,
    };
    Some((sym, false))
}

fn super_digit(c: char) -> char {
    match c {
        '0' => '⁰', '1' => '¹', '2' => '²', '3' => '³', '4' => '⁴',
        '5' => '⁵', '6' => '⁶', '7' => '⁷', '8' => '⁸', '9' => '⁹',
        _ => c,
    }
}

fn sub_digit(c: char) -> char {
    match c {
        '0' => '₀', '1' => '₁', '2' => '₂', '3' => '₃', '4' => '₄',
        '5' => '₅', '6' => '₆', '7' => '₇', '8' => '₈', '9' => '₉',
        _ => c,
    }
}

fn super_char(c: char) -> char {
    match c {
        '+' => '⁺', '-' => '⁻', '=' => '⁼', '(' => '⁽', ')' => '⁾',
        'n' => 'ⁿ', 'i' => 'ⁱ', _ => super_digit(c),
    }
}

fn sub_char(c: char) -> char {
    match c {
        '+' => '₊', '-' => '₋', '=' => '₌', '(' => '₍', ')' => '₎',
        _ => sub_digit(c),
    }
}

/// LaTeX 源 → Unicode 符号串（bounded：见模块注释）。
pub fn latex_to_unicode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\\' {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_alphabetic() {
                j += 1;
            }
            if j == i + 1 {
                // \ 后非字母（\, 等标点转义）——跳过反斜杠保留原字符
                if j < bytes.len() {
                    let ch = input[j..].chars().next().unwrap();
                    out.push(ch);
                    i = j + ch.len_utf8();
                } else {
                    i = j;
                }
                continue;
            }
            let cmd = &input[i + 1..j];
            match cmd_to_unicode(cmd) {
                Some((sym, letterlike)) => {
                    out.push_str(sym);
                    // 字母类符号吞命令后单个空格（TeX 语义：\lambda v → λv）；
                    // 算符/关系保留（a \times b → a × b）。
                    if letterlike
                        && j < bytes.len()
                        && bytes[j] == b' '
                        && j + 1 < bytes.len()
                        && bytes[j + 1].is_ascii_alphanumeric()
                    {
                        i = j + 1;
                    } else {
                        i = j;
                    }
                }
                None => {
                    // 未命中：剥反斜杠保留命令词（\det → det）
                    out.push_str(cmd);
                    i = j;
                }
            }
            continue;
        }
        if b == b'^' || b == b'_' {
            let sup = b == b'^';
            let mut k = i + 1;
            let mut inner = String::new();
            if k < bytes.len() && bytes[k] == b'{' {
                let mut depth = 1usize;
                let mut m = k + 1;
                while m < bytes.len() && depth > 0 {
                    if bytes[m] == b'{' {
                        depth += 1;
                    }
                    if bytes[m] == b'}' {
                        depth -= 1;
                    }
                    m += 1;
                }
                inner = input[k + 1..m.saturating_sub(1)].to_string();
                k = m;
            } else if k < bytes.len() {
                let ch = input[k..].chars().next().unwrap();
                inner.push(ch);
                k += ch.len_utf8();
            }
            if inner.is_empty() {
                // 悬空 ^/_（流式尾）——原样保留
                out.push(if sup { '^' } else { '_' });
            } else if inner.chars().all(|c| c.is_ascii_digit()) {
                out.extend(inner.chars().map(|c| if sup { super_digit(c) } else { sub_digit(c) }));
            } else if inner.chars().count() == 1 {
                let c = inner.chars().next().unwrap();
                out.push(if sup { super_char(c) } else { sub_char(c) });
            } else {
                // 多字符非数字（如 ^{n+1}）——原样保留括号形
                out.push_str(&inner);
            }
            i = k;
            continue;
        }
        let ch = input[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_greek_and_operators() {
        assert_eq!(latex_to_unicode("Av = \\lambda v"), "Av = λv");
        assert_eq!(latex_to_unicode("\\det(A - \\lambda I) = 0"), "det(A - λI) = 0");
        assert_eq!(latex_to_unicode("x^2 + y_1"), "x² + y₁");
        assert_eq!(latex_to_unicode("a \\times b \\pm c"), "a × b ± c");
        assert_eq!(latex_to_unicode("\\alpha \\rightarrow \\omega"), "α → ω");
    }

    #[test]
    fn strips_unknown_commands_and_keeps_pmatrix_shape() {
        assert_eq!(latex_to_unicode("\\begin{pmatrix} 2 & 1 \\end{pmatrix}"), "begin{pmatrix} 2 & 1 end{pmatrix}");
        assert_eq!(latex_to_unicode("E = mc^2"), "E = mc²");
    }

    #[test]
    fn dangling_marker_streaming_safe() {
        assert_eq!(latex_to_unicode("x^"), "x^");
        assert_eq!(latex_to_unicode("x_"), "x_");
    }
}
