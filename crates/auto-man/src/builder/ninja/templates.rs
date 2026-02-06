use std::collections::HashMap;
use super::config::CompilerConfig;

/// 命令模板
pub struct CommandTemplates {
    pub compile: String,
    pub assemble: String,
    pub link: String,
    pub archive: String,
}

impl CommandTemplates {
    /// 为指定的编译器配置创建命令模板
    pub fn new(compiler: &CompilerConfig) -> Self {
        use super::config::CompilerKind;

        match compiler.kind {
            CompilerKind::MSVC => Self::msvc_templates(),
            CompilerKind::GCC => Self::gcc_templates(),
            CompilerKind::Clang => Self::clang_templates(),
            CompilerKind::IAR => Self::iar_templates(),
            CompilerKind::GHS => Self::ghs_templates(),
            CompilerKind::Targeting => Self::targeting_templates(),
            CompilerKind::Hightec => Self::hightec_templates(),
        }
    }

    /// MSVC 命令模板
    fn msvc_templates() -> Self {
        CommandTemplates {
            compile: "{cc} {compile_only} {includes} {defines} {cflags} /Fo{output} {input}".into(),
            assemble: "{as} {includes} {aflags} /Fo{output} {input}".into(),
            link: "{link} /OUT:{output} {input} {ldflags} {libs}".into(),
            archive: "{ar} /OUT:{output} {input}".into(),
        }
    }

    /// GCC 命令模板
    fn gcc_templates() -> Self {
        CommandTemplates {
            compile: "{cc} {compile_only} {includes} {defines} {cflags} -o{output} {input}".into(),
            assemble: "{as} {includes} {aflags} -o{output} {input}".into(),
            link: "{link} -o{output} {input} {ldflags} {libs}".into(),
            archive: "{ar} rcs {output} {input}".into(),
        }
    }

    /// Clang 命令模板
    fn clang_templates() -> Self {
        CommandTemplates {
            compile: "{cc} {compile_only} {includes} {defines} {cflags} -o{output} {input}".into(),
            assemble: "{as} {includes} {aflags} -o{output} {input}".into(),
            link: "{link} -o{output} {input} {ldflags} {libs}".into(),
            archive: "{ar} rcs {output} {input}".into(),
        }
    }

    /// IAR 命令模板
    fn iar_templates() -> Self {
        CommandTemplates {
            compile: "{cc} {includes} {defines} {cflags} --debug -o{output} {input}".into(),
            assemble: "{as} {includes} {aflags} -o{output} {input}".into(),
            link: "{link} -o{output} {input} {ldflags} {libs}".into(),
            archive: "{ar} -o{output} {input}".into(),
        }
    }

    /// GHS 命令模板
    fn ghs_templates() -> Self {
        CommandTemplates {
            compile: "{cc} {compile_only} {includes} {defines} {cflags} -o{output} {input}".into(),
            assemble: "{as} {includes} {aflags} -o{output} {input}".into(),
            link: "{link} -o{output} {input} {ldflags} {libs}".into(),
            archive: "{ar} -r {output} {input}".into(),
        }
    }

    /// Targeting 命令模板
    fn targeting_templates() -> Self {
        CommandTemplates {
            compile: "{cc} {compile_only} {includes} {defines} {cflags} -o{output} {input}".into(),
            assemble: "{as} {includes} {aflags} -o{output} {input}".into(),
            link: "{link} -o{output} {input} {ldflags} {libs}".into(),
            archive: "{ar} -r {output} {input}".into(),
        }
    }

    /// Hightec 命令模板
    fn hightec_templates() -> Self {
        CommandTemplates {
            compile: "{cc} {compile_only} {includes} {defines} {cflags} -o{output} {input}".into(),
            assemble: "{as} {includes} {aflags} -o{output} {input}".into(),
            link: "{link} -o{output} {input} {ldflags} {libs}".into(),
            archive: "{ar} rcs {output} {input}".into(),
        }
    }

    /// 渲染命令模板，替换变量占位符
    ///
    /// # 变量
    /// - `{cc}` - 编译器路径
    /// - `{as}` - 汇编器路径
    /// - `{link}` - 链接器路径
    /// - `{ar}` - 归档器路径
    /// - `{input}` - 输入文件
    /// - `{output}` - 输出文件
    /// - `{includes}` - include 路径
    /// - `{defines}` - 预处理定义
    /// - `{cflags}` - 编译标志
    /// - `{aflags}` - 汇编标志
    /// - `{ldflags}` - 链接标志
    /// - `{libs}` - 库文件
    /// - `{compile_only}` - 仅编译标志
    pub fn render(&self, template: &str, vars: &HashMap<&str, String>) -> String {
        let mut result = template.to_string();

        // 按照变量名长度降序排序，避免部分替换问题
        let mut var_keys: Vec<&&str> = vars.keys().collect();
        var_keys.sort_by_key(|k| std::cmp::Reverse(k.len()));

        for key in var_keys {
            let placeholder = format!("{{{}}}", key);
            if let Some(value) = vars.get(key) {
                result = result.replace(&placeholder, value);
            }
        }

        result
    }

    /// 渲染编译命令
    pub fn render_compile(
        &self,
        cc: &str,
        compile_only: &str,
        includes: &str,
        defines: &str,
        cflags: &str,
        output: &str,
        input: &str,
    ) -> String {
        let mut vars = HashMap::new();
        vars.insert("cc", cc.to_string());
        vars.insert("compile_only", compile_only.to_string());
        vars.insert("includes", includes.to_string());
        vars.insert("defines", defines.to_string());
        vars.insert("cflags", cflags.to_string());
        vars.insert("output", output.to_string());
        vars.insert("input", input.to_string());
        self.render(&self.compile, &vars)
    }

    /// 渲染汇编命令
    pub fn render_assemble(
        &self,
        as_cmd: &str,
        includes: &str,
        aflags: &str,
        output: &str,
        input: &str,
    ) -> String {
        let mut vars = HashMap::new();
        vars.insert("as", as_cmd.to_string());
        vars.insert("includes", includes.to_string());
        vars.insert("aflags", aflags.to_string());
        vars.insert("output", output.to_string());
        vars.insert("input", input.to_string());
        self.render(&self.assemble, &vars)
    }

    /// 渲染链接命令
    pub fn render_link(
        &self,
        link: &str,
        output: &str,
        input: &str,
        ldflags: &str,
        libs: &str,
    ) -> String {
        let mut vars = HashMap::new();
        vars.insert("link", link.to_string());
        vars.insert("output", output.to_string());
        vars.insert("input", input.to_string());
        vars.insert("ldflags", ldflags.to_string());
        vars.insert("libs", libs.to_string());
        self.render(&self.link, &vars)
    }

    /// 渲染归档命令
    pub fn render_archive(&self, ar: &str, output: &str, input: &str) -> String {
        let mut vars = HashMap::new();
        vars.insert("ar", ar.to_string());
        vars.insert("output", output.to_string());
        vars.insert("input", input.to_string());
        self.render(&self.archive, &vars)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_template() {
        let templates = CommandTemplates::gcc_templates();
        let template = "{cc} {compile_only} -o{output} {input}";
        let mut vars = HashMap::new();
        vars.insert("cc", "gcc".to_string());
        vars.insert("compile_only", "-c".to_string());
        vars.insert("output", "main.o".to_string());
        vars.insert("input", "main.c".to_string());

        let result = templates.render(&template, &vars);
        assert_eq!(result, "gcc -c -omain.o main.c");
    }

    #[test]
    fn test_msvc_templates() {
        let templates = CommandTemplates::msvc_templates();
        assert!(templates.compile.contains("{cc}"));
        assert!(templates.compile.contains("/Fo{output}"));
        assert!(templates.link.contains("/OUT:{output}"));
        assert!(templates.archive.contains("/OUT:{output}"));
    }

    #[test]
    fn test_gcc_templates() {
        let templates = CommandTemplates::gcc_templates();
        assert!(templates.compile.contains("{cc}"));
        assert!(templates.compile.contains("-o{output}"));
        assert!(templates.link.contains("-o{output}"));
        assert!(templates.archive.contains("rcs {output}"));
    }

    #[test]
    fn test_render_compile() {
        let templates = CommandTemplates::gcc_templates();
        let result = templates.render_compile(
            "gcc",
            "-c",
            "-Iinclude",
            "-DDEBUG",
            "-O2",
            "main.o",
            "main.c"
        );
        assert_eq!(
            result,
            "gcc -c -Iinclude -DDEBUG -O2 -omain.o main.c"
        );
    }

    #[test]
    fn test_render_link() {
        let templates = CommandTemplates::gcc_templates();
        let result = templates.render_link(
            "gcc",
            "main",
            "main.o",
            "",
            "-lmylib"
        );
        assert_eq!(
            result,
            "gcc -omain main.o  -lmylib"
        );
    }

    #[test]
    fn test_render_archive() {
        let templates = CommandTemplates::gcc_templates();
        let result = templates.render_archive(
            "ar",
            "libmylib.a",
            "obj1.o obj2.o"
        );
        assert_eq!(
            result,
            "ar rcs libmylib.a obj1.o obj2.o"
        );
    }
}
