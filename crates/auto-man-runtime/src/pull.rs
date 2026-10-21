#[cfg(test)]
mod tests {
    use auto_lang_core::parse;
    use auto_lang_core::util::pretty;

    #[test]
    fn test_read_config() {
        let config = r#"
            name: "hello"
            version: "0.1.0"

            exe("hello") {
                dir: "src"
                main: "main.c"
            }
        "#;

        let auto = parse(config).unwrap();
        println!("{}", auto);
        println!("{}", pretty(&auto.to_string()));
    }
}
