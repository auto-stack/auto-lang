//! AutoDown Parser - Flip Mechanism
//!
//! Implements the Flip mechanism for switching between text and code modes.

use super::ast::*;
use super::error::{AdocError, AdocResult};
use super::lexer::{AdToken, AdTokenKind, AdocLexer, LexerMode};

/// AutoDown Parser with Flip mechanism
pub struct AdocParser<'a> {
    /// Lexer
    lexer: AdocLexer<'a>,

    /// Current token
    current: AdToken,

    /// Peek token
    peek: Option<AdToken>,

    /// Parser mode
    mode: ParserMode,

    /// Current section stack
    section_stack: Vec<AdocSection>,
}

/// Parser mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserMode {
    /// Document level (sections, blocks)
    Document,
    /// Inside section content
    Section,
    /// Inside code block ($...)
    Code,
    /// Inside math block (%{...})
    Math,
}

impl<'a> AdocParser<'a> {
    /// Create a new parser
    pub fn new(source: &'a str) -> Self {
        let mut lexer = AdocLexer::new(source);
        let current = lexer
            .next_token()
            .unwrap_or_else(|_| AdToken::simple(AdTokenKind::EOF, 1, 1));

        Self {
            lexer,
            current,
            peek: None,
            mode: ParserMode::Document,
            section_stack: Vec::new(),
        }
    }

    /// Parse a complete document
    pub fn parse(&mut self) -> AdocResult<AdocDocument> {
        let mut doc = AdocDocument::default();
        let mut current_section: Option<AdocSection> = None;

        loop {
            match self.current.kind.clone() {
                AdTokenKind::EOF => break,

                AdTokenKind::Header { level } => {
                    let title = self.parse_header_title()?;

                    if let Some(section) = current_section.take() {
                        self.add_section_to_document(&mut doc, section)?;
                    }

                    let mut section = AdocSection::new(level, title);

                    // Parse section content
                    self.advance()?;
                    while !matches!(
                        self.current.kind,
                        AdTokenKind::EOF | AdTokenKind::Header { .. }
                    ) {
                        if let Some(block) = self.parse_block()? {
                            section.content.push(block);
                        } else {
                            self.advance()?; // Advance for unrecognized tokens (e.g., BlankLine)
                        }
                    }

                    current_section = Some(section);
                }

                _ => {
                    // Preamble content (before first header)
                    if let Some(block) = self.parse_block()? {
                        doc.preamble.push(block);
                    } else {
                        self.advance()?;
                    }
                }
            }
        }

        // Add final section
        if let Some(section) = current_section {
            self.add_section_to_document(&mut doc, section)?;
        }

        Ok(doc)
    }

    /// Add section to document, handling nesting
    fn add_section_to_document(
        &self,
        doc: &mut AdocDocument,
        section: AdocSection,
    ) -> AdocResult<()> {
        // For simplicity, we'll just add sections flat for now
        // TODO: Implement proper nesting based on levels
        doc.sections.push(section);
        Ok(())
    }

    /// Parse header title
    fn parse_header_title(&mut self) -> AdocResult<String> {
        let mut title_parts = Vec::new();

        self.advance()?; // Skip header token

        loop {
            match &self.current.kind {
                AdTokenKind::Newline | AdTokenKind::BlankLine | AdTokenKind::EOF => {
                    break;
                }
                _ => {
                    title_parts.push(self.current.text.clone());
                    self.advance()?;
                }
            }
        }

        Ok(title_parts.join(" "))
    }

    /// Parse a block
    fn parse_block(&mut self) -> AdocResult<Option<AdocBlock>> {
        match &self.current.kind {
            AdTokenKind::BlankLine | AdTokenKind::Newline => {
                // Return None without advancing - let the main loop handle it
                // to avoid double-advance
                Ok(None)
            }

            AdTokenKind::Text => self.parse_paragraph(),

            // Inline markup can start a paragraph
            AdTokenKind::StarStar => self.parse_paragraph(),
            AdTokenKind::Underscore => self.parse_paragraph(),
            AdTokenKind::Backtick => self.parse_paragraph(),
            AdTokenKind::LinkStart => self.parse_paragraph(),
            AdTokenKind::ImageStart => self.parse_paragraph(),

            AdTokenKind::Dollar => self.parse_code_block(),


            AdTokenKind::InterpolateStart => self.parse_interpolation_as_block(),


            AdTokenKind::MathStart => self.parse_math_block(),


            AdTokenKind::ListItem => self.parse_list(),


            AdTokenKind::NumberedList => self.parse_numbered_list(),


            AdTokenKind::CodeFence => self.parse_fenced_code(),


            AdTokenKind::Blockquote => self.parse_blockquote(),


            AdTokenKind::HorizontalRule => {
                self.advance()?;
                Ok(Some(AdocBlock::HorizontalRule))
            }

            _ => {
                // Unknown token, skip
                self.advance()?;
                Ok(None)
            }
        }
    }


    /// Parse a paragraph
    fn parse_paragraph(&mut self) -> AdocResult<Option<AdocBlock>> {
        let mut inlines = Vec::new();


        loop {
            match &self.current.kind {
                AdTokenKind::Text => {
                    inlines.push(AdocInline::Text(self.current.text.clone()));
                    self.advance()?;
                }


                AdTokenKind::StarStar => {
                    self.advance()?;
                    self.advance()?;
                    let bold_content = self.parse_inline_until(AdTokenKind::StarStar)?;
                    self.advance()?; // consume closing **
                    inlines.push(AdocInline::Bold(bold_content));
                }

                AdTokenKind::Underscore => {
                    self.advance()?;
                    let italic_content = self.parse_inline_until(AdTokenKind::Underscore)?;
                    self.advance()?; // consume closing _
                    inlines.push(AdocInline::Italic(italic_content));
                }

                AdTokenKind::Backtick => {
                    self.advance()?;
                    let mut code = String::new();
                    while !matches!(self.current.kind, AdTokenKind::Backtick | AdTokenKind::EOF) {
                        code.push_str(&self.current.text);
                        self.advance()?;
                    }
                    self.advance()?; // consume closing `
                    inlines.push(AdocInline::Code(code));
                }

                AdTokenKind::MathStart => {
                    self.advance()?; // consume %{
                    let math = self.parse_math_content()?;
                    inlines.push(AdocInline::Math(math));
                }

                AdTokenKind::InterpolateStart => {
                    self.advance()?; // consume ${
                    let expr = self.parse_expression()?;
                    inlines.push(AdocInline::Interpolate(expr));
                }

                AdTokenKind::LinkStart => {
                    let link = self.parse_link()?;
                    inlines.push(link);
                }

                AdTokenKind::ImageStart => {
                    let image = self.parse_image()?;
                    inlines.push(image);
                }

                AdTokenKind::Newline | AdTokenKind::BlankLine | AdTokenKind::EOF => {
                    break;
                }

                _ => {
                    self.advance()?;
                }
            }
        }

        if inlines.is_empty() {
            Ok(None)
        } else {
            Ok(Some(AdocBlock::Paragraph(inlines)))
        }
    }

    fn parse_inline_until(&mut self, end: AdTokenKind) -> AdocResult<Vec<AdocInline>> {
        let mut content = Vec::new();

        while self.current.kind != end && self.current.kind != AdTokenKind::EOF {
            match &self.current.kind {
                AdTokenKind::Text => {
                    content.push(AdocInline::Text(self.current.text.clone()));
                }
                _ => {
                    // For simplicity, treat other tokens as text
                    content.push(AdocInline::Text(self.current.text.clone()));
                }
            }
            self.advance()?;
        }

        Ok(content)
    }

    /// Parse a link [text](url)
    fn parse_link(&mut self) -> AdocResult<AdocInline> {
        // [ already consumed
        let mut text = String::new();

        while !matches!(self.current.kind, AdTokenKind::RBracket | AdTokenKind::EOF) {
            text.push_str(&self.current.text);
            self.advance()?;
        }

        self.advance()?; // consume ]

        // Expect (url)
        if self.current.kind == AdTokenKind::LParen {
            self.advance()?;
            let mut url = String::new();

            while !matches!(self.current.kind, AdTokenKind::RParen | AdTokenKind::EOF) {
                url.push_str(&self.current.text);
                self.advance()?;
            }

            self.advance()?; // consume )

            Ok(AdocInline::Link { text, url })
        } else {
            Err(AdocError::parser("Expected ( after link text"))
        }
    }

    /// Parse an image ![alt](url)
    fn parse_image(&mut self) -> AdocResult<AdocInline> {
        // ![ already consumed, [ was consumed too
        let mut alt = String::new();

        while !matches!(self.current.kind, AdTokenKind::RBracket | AdTokenKind::EOF) {
            alt.push_str(&self.current.text);
            self.advance()?;
        }

        self.advance()?; // consume ]

        // Expect (url)
        if self.current.kind == AdTokenKind::LParen {
            self.advance()?;
            let mut url = String::new();

            while !matches!(self.current.kind, AdTokenKind::RParen | AdTokenKind::EOF) {
                url.push_str(&self.current.text);
                self.advance()?;
            }

            self.advance()?; // consume )

            Ok(AdocInline::Image { alt, url })
        } else {
            Err(AdocError::parser("Expected ( after image alt text"))
        }
    }

    /// Parse math content (after %{)
    fn parse_math_content(&mut self) -> AdocResult<AdocMath> {
        let mut content = String::new();
        let mut is_display = false;

        // Check if this is display math (starts with newline)
        if self.current.kind == AdTokenKind::Newline {
            is_display = true;
            self.advance()?;
        }

        loop {
            match &self.current.kind {
                AdTokenKind::MathContent => {
                    content.push_str(&self.current.text);
                    self.advance()?;
                }
                AdTokenKind::MathEnd => {
                    self.advance()?;
                    break;
                }
                AdTokenKind::EOF => {
                    return Err(AdocError::unterminated("math block", 1));
                }
                _ => {
                    content.push_str(&self.current.text);
                    self.advance()?;
                }
            }
        }

        let content = content.trim().to_string();

        if is_display {
            Ok(AdocMath::display(content))
        } else {
            Ok(AdocMath::inline(content))
        }
    }

    /// Parse a math block
    fn parse_math_block(&mut self) -> AdocResult<Option<AdocBlock>> {
        self.advance()?; // consume %{

        let math = self.parse_math_content()?;
        Ok(Some(AdocBlock::MathBlock(math)))
    }

    /// Parse a code block (after $)
    fn parse_code_block(&mut self) -> AdocResult<Option<AdocBlock>> {
        self.advance()?; // consume $

        // Check for control flow keywords
        match &self.current.kind {
            AdTokenKind::If => self.parse_if_block(),
            AdTokenKind::For => self.parse_for_block(),
            AdTokenKind::Ident => {
                // Component call
                self.parse_component_call()
            }
            _ => {
                // Raw code block
                let code = self.parse_raw_code()?;
                Ok(Some(AdocBlock::RawCode(code)))
            }
        }
    }

    /// Parse an if block
    fn parse_if_block(&mut self) -> AdocResult<Option<AdocBlock>> {
        self.advance()?; // consume 'if'

        // Parse condition
        let condition = self.parse_condition()?;

        // Expect {
        if self.current.kind != AdTokenKind::LBrace {
            return Err(AdocError::unexpected_token("{", &self.current.text));
        }
        self.advance()?;

        // Flip to text mode and parse then body
        self.lexer.set_mode(LexerMode::Text);
        let mut then_body = Vec::new();

        // Simple parsing: collect until $else or closing }
        let mut brace_level = 1;

        loop {
            match &self.current.kind {
                AdTokenKind::Dollar => {
                    // Check for $else
                    self.advance()?;
                    if self.current.kind == AdTokenKind::Else {
                        break;
                    }
                    // Not $else, continue
                    then_body.push(AdocBlock::RawCode("$".to_string()));
                }
                AdTokenKind::LBrace => {
                    brace_level += 1;
                    // Treat as text
                    if let Some(block) = self.parse_block()? {
                        then_body.push(block);
                    }
                }
                AdTokenKind::RBrace => {
                    brace_level -= 1;
                    if brace_level == 0 {
                        self.advance()?;
                        break;
                    }
                    if let Some(block) = self.parse_block()? {
                        then_body.push(block);
                    }
                }
                AdTokenKind::EOF => {
                    return Err(AdocError::unterminated("if block", 1));
                }
                _ => {
                    if let Some(block) = self.parse_block()? {
                        then_body.push(block);
                    }
                }
            }
        }

        // Check for $else
        let mut else_body = None;
        if self.current.kind == AdTokenKind::Else {
            self.advance()?;

            // Expect {
            if self.current.kind != AdTokenKind::LBrace {
                return Err(AdocError::unexpected_token("{", &self.current.text));
            }
            self.advance()?;

            // Parse else body
            let mut else_blocks = Vec::new();
            brace_level = 1;

            loop {
                match &self.current.kind {
                    AdTokenKind::LBrace => {
                        brace_level += 1;
                        if let Some(block) = self.parse_block()? {
                            else_blocks.push(block);
                        }
                    }
                    AdTokenKind::RBrace => {
                        brace_level -= 1;
                        if brace_level == 0 {
                            self.advance()?;
                            break;
                        }
                        if let Some(block) = self.parse_block()? {
                            else_blocks.push(block);
                        }
                    }
                    AdTokenKind::EOF => {
                        return Err(AdocError::unterminated("else block", 1));
                    }
                    _ => {
                        if let Some(block) = self.parse_block()? {
                            else_blocks.push(block);
                        }
                    }
                }
            }

            else_body = Some(else_blocks);
        }

        Ok(Some(AdocBlock::If {
            condition,
            then_body,
            else_body,
        }))
    }

    /// Parse a for block
    fn parse_for_block(&mut self) -> AdocResult<Option<AdocBlock>> {
        self.advance()?; // consume 'for'

        // Parse variable name
        if self.current.kind != AdTokenKind::Ident {
            return Err(AdocError::unexpected_token(
                "identifier",
                &self.current.text,
            ));
        }
        let var = self.current.text.clone();
        self.advance()?;

        // Expect 'in'
        if self.current.kind != AdTokenKind::In {
            return Err(AdocError::unexpected_token("in", &self.current.text));
        }
        self.advance()?;

        // Parse iterable (simplified)
        let mut iterable = String::new();
        while self.current.kind != AdTokenKind::LBrace {
            iterable.push_str(&self.current.text);
            self.advance()?;
        }

        // Expect {
        if self.current.kind != AdTokenKind::LBrace {
            return Err(AdocError::unexpected_token("{", &self.current.text));
        }
        self.advance()?;

        // Flip to text mode and parse body
        self.lexer.set_mode(LexerMode::Text);
        let mut body = Vec::new();
        let mut brace_level = 1;

        loop {
            match &self.current.kind {
                AdTokenKind::LBrace => {
                    brace_level += 1;
                    if let Some(block) = self.parse_block()? {
                        body.push(block);
                    }
                }
                AdTokenKind::RBrace => {
                    brace_level -= 1;
                    if brace_level == 0 {
                        self.advance()?;
                        break;
                    }
                    if let Some(block) = self.parse_block()? {
                        body.push(block);
                    }
                }
                AdTokenKind::EOF => {
                    return Err(AdocError::unterminated("for block", 1));
                }
                _ => {
                    if let Some(block) = self.parse_block()? {
                        body.push(block);
                    }
                }
            }
        }

        Ok(Some(AdocBlock::For {
            var,
            index: None,
            iterable,
            body,
        }))
    }

    /// Parse a component call
    fn parse_component_call(&mut self) -> AdocResult<Option<AdocBlock>> {
        let name = self.current.text.clone();
        self.advance()?;

        let mut props = std::collections::HashMap::new();

        // Check for props (...)
        if self.current.kind == AdTokenKind::LParen {
            self.advance()?;

            while self.current.kind != AdTokenKind::RParen {
                // Parse prop: value pairs
                if self.current.kind == AdTokenKind::Ident {
                    let key = self.current.text.clone();
                    self.advance()?;

                    if self.current.kind == AdTokenKind::Colon {
                        self.advance()?;
                        let value = self.parse_expression()?;
                        props.insert(key, value);
                    }

                    // Skip comma
                    if self.current.kind == AdTokenKind::Comma {
                        self.advance()?;
                    }
                } else {
                    self.advance()?;
                }
            }
            self.advance()?; // consume )
        }

        // Check for trailing closure { ... }
        let mut children = Vec::new();
        if self.current.kind == AdTokenKind::LBrace {
            self.advance()?;

            // Flip to text mode
            self.lexer.set_mode(LexerMode::Text);
            let mut brace_level = 1;

            loop {
                match &self.current.kind {
                    AdTokenKind::LBrace => {
                        brace_level += 1;
                        if let Some(block) = self.parse_block()? {
                            children.push(block);
                        }
                    }
                    AdTokenKind::RBrace => {
                        brace_level -= 1;
                        if brace_level == 0 {
                            self.advance()?;
                            break;
                        }
                        if let Some(block) = self.parse_block()? {
                            children.push(block);
                        }
                    }
                    AdTokenKind::EOF => {
                        return Err(AdocError::unterminated("component block", 1));
                    }
                    _ => {
                        if let Some(block) = self.parse_block()? {
                            children.push(block);
                        }
                    }
                }
            }
        }

        Ok(Some(AdocBlock::Component {
            name,
            props,
            children,
        }))
    }

    /// Parse a condition expression
    fn parse_condition(&mut self) -> AdocResult<String> {
        let mut condition = String::new();

        while !matches!(self.current.kind, AdTokenKind::LBrace | AdTokenKind::EOF) {
            condition.push_str(&self.current.text);
            condition.push(' ');
            self.advance()?;
        }

        Ok(condition.trim().to_string())
    }

    /// Parse raw code (until closing })
    fn parse_raw_code(&mut self) -> AdocResult<String> {
        let mut code = String::new();

        while !matches!(self.current.kind, AdTokenKind::RBrace | AdTokenKind::EOF) {
            code.push_str(&self.current.text);
            code.push(' ');
            self.advance()?;
        }

        Ok(code.trim().to_string())
    }

    /// Parse an expression (for interpolation)
    fn parse_expression(&mut self) -> AdocResult<AdocExpr> {
        // Simplified expression parsing
        match &self.current.kind {
            AdTokenKind::String => {
                let s = self.current.text.clone();
                self.advance()?;
                Ok(AdocExpr::Literal(s))
            }
            AdTokenKind::Number => {
                let num_str = self.current.text.clone();
                self.advance()?;
                if num_str.contains('.') {
                    Ok(AdocExpr::Float(num_str.parse().unwrap_or(0.0)))
                } else {
                    Ok(AdocExpr::Int(num_str.parse().unwrap_or(0)))
                }
            }
            AdTokenKind::True => {
                self.advance()?;
                Ok(AdocExpr::Bool(true))
            }
            AdTokenKind::False => {
                self.advance()?;
                Ok(AdocExpr::Bool(false))
            }
            AdTokenKind::Nil => {
                self.advance()?;
                Ok(AdocExpr::Literal("nil".to_string()))
            }
            AdTokenKind::Ident => {
                let name = self.current.text.clone();
                self.advance()?;

                // Check for property access
                let mut expr = AdocExpr::Var(name);

                while self.current.kind == AdTokenKind::Dot {
                    self.advance()?;
                    if self.current.kind == AdTokenKind::Ident {
                        let prop = self.current.text.clone();
                        self.advance()?;
                        expr = AdocExpr::Property {
                            object: Box::new(expr),
                            property: prop,
                        };
                    }
                }

                // Check for method call
                if self.current.kind == AdTokenKind::LParen {
                    self.advance()?;
                    let mut args = Vec::new();

                    while self.current.kind != AdTokenKind::RParen {
                        args.push(self.parse_expression()?);
                        if self.current.kind == AdTokenKind::Comma {
                            self.advance()?;
                        }
                    }
                    self.advance()?;

                    // Convert to method call if it's a property
                    if let AdocExpr::Property { object, property } = expr {
                        expr = AdocExpr::MethodCall {
                            object,
                            method: property,
                            args,
                        };
                    }
                }

                Ok(expr)
            }
            _ => Err(AdocError::invalid_expression(&format!(
                "unexpected token: {:?}",
                self.current.kind
            ))),
        }
    }

    /// Parse interpolation as block (for code-like contexts)
    fn parse_interpolation_as_block(&mut self) -> AdocResult<Option<AdocBlock>> {
        self.advance()?; // consume ${

        let expr = self.parse_expression()?;

        // Expect }
        if self.current.kind == AdTokenKind::RBrace {
            self.advance()?;
        }

        // Wrap in paragraph for now
        Ok(Some(AdocBlock::Paragraph(vec![AdocInline::Interpolate(
            expr,
        )])))
    }

    /// Parse a list
    fn parse_list(&mut self) -> AdocResult<Option<AdocBlock>> {
        let mut items = Vec::new();

        while self.current.kind == AdTokenKind::ListItem {
            self.advance()?; // consume -

            let mut content = Vec::new();
            while !matches!(
                self.current.kind,
                AdTokenKind::Newline
                    | AdTokenKind::BlankLine
                    | AdTokenKind::ListItem
                    | AdTokenKind::EOF
            ) {
                match &self.current.kind {
                    AdTokenKind::Text => {
                        content.push(AdocInline::Text(self.current.text.clone()));
                    }
                    AdTokenKind::InterpolateStart => {
                        self.advance()?;
                        let expr = self.parse_expression()?;
                        content.push(AdocInline::Interpolate(expr));
                        continue;
                    }
                    _ => {}
                }
                self.advance()?;
            }

            items.push(AdocListItem {
                content,
                nested: None,
            });

            // Skip newline
            if self.current.kind == AdTokenKind::Newline {
                self.advance()?;
            }
        }

        if items.is_empty() {
            Ok(None)
        } else {
            Ok(Some(AdocBlock::List {
                items,
                ordered: false,
            }))
        }
    }

    /// Parse a numbered list
    fn parse_numbered_list(&mut self) -> AdocResult<Option<AdocBlock>> {
        let mut items = Vec::new();

        while self.current.kind == AdTokenKind::NumberedList {
            self.advance()?; // consume number.

            let mut content = Vec::new();
            while !matches!(
                self.current.kind,
                AdTokenKind::Newline
                    | AdTokenKind::BlankLine
                    | AdTokenKind::NumberedList
                    | AdTokenKind::EOF
            ) {
                if self.current.kind == AdTokenKind::Text {
                    content.push(AdocInline::Text(self.current.text.clone()));
                }
                self.advance()?;
            }

            items.push(AdocListItem {
                content,
                nested: None,
            });

            // Skip newline
            if self.current.kind == AdTokenKind::Newline {
                self.advance()?;
            }
        }

        if items.is_empty() {
            Ok(None)
        } else {
            Ok(Some(AdocBlock::List {
                items,
                ordered: true,
            }))
        }
    }

    /// Parse fenced code block
    fn parse_fenced_code(&mut self) -> AdocResult<Option<AdocBlock>> {
        self.advance()?; // consume ```

        // Get language (if any)
        let lang = if self.current.kind == AdTokenKind::Text {
            let lang = Some(self.current.text.clone());
            self.advance()?;
            lang
        } else {
            None
        };

        // Skip newline
        if self.current.kind == AdTokenKind::Newline {
            self.advance()?;
        }

        // Collect code until closing ```
        let mut code = String::new();

        loop {
            match &self.current.kind {
                AdTokenKind::CodeFence => {
                    self.advance()?;
                    break;
                }
                AdTokenKind::EOF => {
                    return Err(AdocError::unterminated("code fence", 1));
                }
                AdTokenKind::Newline => {
                    code.push('\n');
                    self.advance()?;
                }
                _ => {
                    code.push_str(&self.current.text);
                    self.advance()?;
                }
            }
        }

        Ok(Some(AdocBlock::CodeBlock {
            lang,
            code: code.trim().to_string(),
        }))
    }

    /// Parse blockquote
    fn parse_blockquote(&mut self) -> AdocResult<Option<AdocBlock>> {
        self.advance()?; // consume >

        let mut content = Vec::new();

        loop {
            match &self.current.kind {
                AdTokenKind::Newline | AdTokenKind::BlankLine | AdTokenKind::EOF => {
                    break;
                }
                AdTokenKind::Blockquote => {
                    // Nested blockquote
                    self.advance()?;
                }
                _ => {
                    if let Some(block) = self.parse_block()? {
                        content.push(block);
                    }
                }
            }
        }

        if content.is_empty() {
            Ok(None)
        } else {
            Ok(Some(AdocBlock::Blockquote(content)))
        }
    }

    /// Advance to next token
    fn advance(&mut self) -> AdocResult<()> {
        if let Some(token) = self.peek.take() {
            self.current = token;
        } else {
            self.current = self.lexer.next_token()?;
        }

        // Pre-load peek
        self.peek = Some(self.lexer.next_token()?);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_document() {
        let source = r#"# Title

This is a paragraph.

## Section

Another paragraph.
"#;

        let mut parser = AdocParser::new(source);
        let doc = parser.parse().unwrap();

        assert_eq!(doc.sections.len(), 2);
    }

    #[test]
    fn test_parse_math() {
        let source = r#"Math: %{ E = mc^2 }"#;

        let mut parser = AdocParser::new(source);
        let doc = parser.parse().unwrap();

        assert_eq!(doc.preamble.len(), 1);
        if let Some(block) = doc.preamble.first() {
            if let AdocBlock::Paragraph(inlines) = block {
                assert!(inlines.iter().any(|i| matches!(i, AdocInline::Math(_))));
            }
        }
    }

    #[test]
    fn test_parse_list() {
        let source = r#"- Item 1
- Item 2
- Item 3
"#;

        let mut parser = AdocParser::new(source);
        let doc = parser.parse().unwrap();

        assert_eq!(doc.preamble.len(), 1);
        if let Some(block) = doc.preamble.first() {
            if let AdocBlock::List { items, ordered } = block {
                assert!(!ordered);
                assert_eq!(items.len(), 3);
            }
        }
    }
}
