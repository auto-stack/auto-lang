// Helper function to parse generic type instance in expression context
// Returns a GenName expression with the formatted generic type name (e.g., "List<int>")
fn parse_generic_instance_expr(&mut self, base_name: Name) -> AutoResult<Expr> {
    use crate::ast::Type;

    self.expect(TokenKind::Lt)?;

    let mut args = Vec::new();
    args.push(self.parse_type()?);

    while self.cur.kind == TokenKind::Comma {
        self.next(); // Consume ','
        args.push(self.parse_type()?);
    }

    self.expect(TokenKind::Gt)?;

    // Generate a descriptive name for the generic instance
    // e.g., "List<int>", "Heap<T>", "Map<str, int>"
    let args_str = args.iter()
        .map(|t| t.unique_name().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let generic_name = format!("{}<{}>", base_name, args_str);

    // Use GenName to represent generic type instances in expressions
    Ok(Expr::GenName(generic_name.into()))
}
