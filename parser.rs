//Ryan Domathoti
// Recursive-descent syntax analyzer (Task 2). One method per grammar rule,
// one token of lookahead (`current`). The AST is built with a stack of frames:
// each block pushes a frame, its children land in it, and on `pop` they are
// wrapped into a single node that goes into the enclosing frame.

use crate::ast::Node;
use crate::diagnostics::Reporter;
use crate::lexer::EOF;
use crate::traits::SyntaxAnalyzer;

pub struct Parser {
    tokens: Vec<String>,
    positions: Vec<(usize, usize)>,
    idx: usize,
    current: String,
    frames: Vec<Vec<Node>>,
    root: Option<Node>,
    reporter: Reporter,
}

fn is_text(tok: &str) -> bool {
    !tok.is_empty() && !tok.starts_with('#')
}

impl Parser {
    pub fn new(reporter: Reporter) -> Self {
        Parser {
            tokens: vec![EOF.to_string()],
            positions: vec![(1, 1)],
            idx: 0,
            current: EOF.to_string(),
            frames: Vec::new(),
            root: None,
            reporter,
        }
    }

    /// Hand the parser a fresh token list (must end with EOF) and reset it.
    pub fn load(&mut self, tokens: Vec<String>, positions: Vec<(usize, usize)>) {
        self.current = tokens.first().cloned().unwrap_or_default();
        self.tokens = tokens;
        self.positions = positions;
        self.idx = 0;
        self.frames.clear();
        self.root = None;
    }

    pub fn current(&self) -> &str {
        &self.current
    }

    pub fn set_current(&mut self, tok: String) {
        self.current = tok;
    }

    /// Take the finished tree (only present after a successful parse).
    pub fn take_tree(&mut self) -> Option<Node> {
        self.root.take()
    }

    // ---------- helpers ----------

    fn advance(&mut self) {
        if self.idx + 1 < self.tokens.len() {
            self.idx += 1;
        }
        self.current = self.tokens[self.idx].clone();
    }

    fn describe(tok: &str) -> String {
        if tok.is_empty() { "end of file".to_string() } else { format!("'{}'", tok) }
    }

    fn syntax_err(&self, msg: &str) -> String {
        let (line, col) = self.positions[self.idx.min(self.positions.len() - 1)];
        let len = self.current.chars().count();
        self.reporter.render("Syntax error", line, col, len, msg)
    }

    fn expect(&mut self, tok: &str) -> Result<(), String> {
        if self.current == tok {
            self.advance();
            Ok(())
        } else {
            let msg = format!("expected '{}' but found {}", tok, Self::describe(&self.current));
            Err(self.syntax_err(&msg))
        }
    }

    fn push_frame(&mut self) {
        self.frames.push(Vec::new());
    }

    fn pop_frame(&mut self) -> Vec<Node> {
        self.frames.pop().unwrap_or_default()
    }

    fn push_node(&mut self, n: Node) {
        if let Some(top) = self.frames.last_mut() {
            top.push(n);
        }
    }

    fn at_body_item(&self) -> bool {
        matches!(
            self.current.as_str(),
            "#MAEK PARAGRAF" | "#GIMMEH BOLD" | "#MAEK LIST" | "#NEWLINE" | "#GIMMEH LINX" | "#GIMMEH VAR"
        ) || is_text(&self.current)
    }

    fn at_inner_text(&self) -> bool {
        // Same as a body item, minus paragraph (no nesting), plus comment.
        (self.at_body_item() && self.current != "#MAEK PARAGRAF") || self.current == "#OBTW"
    }

    /// A single word of text becomes a Text node.
    fn parse_word(&mut self) -> Result<(), String> {
        if !is_text(&self.current) {
            let msg = format!("expected text but found {}", Self::describe(&self.current));
            return Err(self.syntax_err(&msg));
        }
        let w = self.current.clone();
        self.advance();
        self.push_node(Node::Text(w));
        Ok(())
    }

    fn parse_define_opt(&mut self) -> Result<(), String> {
        if self.current == "#IHAZ" {
            self.parse_variable_define()?;
        }
        Ok(())
    }

    fn parse_body_item(&mut self) -> Result<(), String> {
        let tok = self.current.clone();
        match tok.as_str() {
            "#MAEK PARAGRAF" => self.parse_paragraph(),
            "#GIMMEH BOLD" => self.parse_bold(),
            "#MAEK LIST" => self.parse_list(),
            "#NEWLINE" => self.parse_newline(),
            "#GIMMEH LINX" => self.parse_link(),
            "#GIMMEH VAR" => self.parse_variable_use(),
            _ => self.parse_word(),
        }
    }

    fn parse_item(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH ITEM")?;
        self.push_frame();
        self.parse_define_opt()?;
        loop {
            let tok = self.current.clone();
            match tok.as_str() {
                "#GIMMEH BOLD" => self.parse_bold()?,
                "#GIMMEH VAR" => self.parse_variable_use()?,
                "#OBTW" => self.parse_comment()?,
                t if is_text(t) => self.parse_word()?,
                _ => break,
            }
        }
        self.expect("#OIC")?;
        let kids = self.pop_frame();
        self.push_node(Node::Item(kids));
        Ok(())
    }

    /// <after_hai>: comments, then optional head, then variable define / body.
    fn parse_after_hai(&mut self) -> Result<(), String> {
        while self.current == "#OBTW" {
            self.parse_comment()?;
        }
        if self.current == "#MAEK HEAD" {
            self.parse_head()?;
            self.parse_after_head()
        } else {
            self.parse_after_head()
        }
    }

    /// <after_head>: comments, then variable define / body.
    fn parse_after_head(&mut self) -> Result<(), String> {
        while self.current == "#OBTW" {
            self.parse_comment()?;
        }
        if self.current == "#IHAZ" {
            self.parse_variable_define()?;
        }
        self.parse_body()
    }
}

impl SyntaxAnalyzer for Parser {
    fn parse_lolcode(&mut self) -> Result<(), String> {
        self.frames.clear();
        self.push_frame();
        self.expect("#HAI")?;
        self.parse_after_hai()?;
        self.expect("#KBYE")?;
        if self.current != EOF {
            let msg = format!("unexpected {} after #KBYE", Self::describe(&self.current));
            return Err(self.syntax_err(&msg));
        }
        let kids = self.pop_frame();
        self.root = Some(Node::Html(kids));
        Ok(())
    }

    fn parse_head(&mut self) -> Result<(), String> {
        self.expect("#MAEK HEAD")?;
        self.push_frame();
        self.parse_define_opt()?;
        while self.current == "#OBTW" {
            self.parse_comment()?;
        }
        self.parse_title()?;
        while self.current == "#OBTW" {
            self.parse_comment()?;
        }
        self.expect("#MKAY")?;
        let kids = self.pop_frame();
        self.push_node(Node::Head(kids));
        Ok(())
    }

    fn parse_title(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH TITLE")?;
        self.push_frame();
        self.parse_text()?;
        let mut kids = self.pop_frame();
        self.expect("#OIC")?;
        let text = match kids.pop() {
            Some(Node::Text(t)) => t,
            _ => String::new(),
        };
        self.push_node(Node::Title(text));
        Ok(())
    }

    fn parse_comment(&mut self) -> Result<(), String> {
        self.expect("#OBTW")?;
        let mut words: Vec<String> = Vec::new();
        while is_text(&self.current) {
            words.push(self.current.clone());
            self.advance();
        }
        self.expect("#TLDR")?;
        self.push_node(Node::Comment(words.join(" ")));
        Ok(())
    }

    fn parse_body(&mut self) -> Result<(), String> {
        // <body> is right-recursive in the BNF; a loop is equivalent.
        loop {
            if self.current == "#OBTW" {
                self.parse_comment()?;
            } else if self.at_body_item() {
                self.parse_body_item()?;
            } else {
                return Ok(());
            }
        }
    }

    fn parse_paragraph(&mut self) -> Result<(), String> {
        self.expect("#MAEK PARAGRAF")?;
        self.push_frame();
        self.parse_define_opt()?;
        self.parse_inner_paragraph()?;
        self.expect("#MKAY")?;
        let kids = self.pop_frame();
        self.push_node(Node::Paragraph(kids));
        Ok(())
    }

    fn parse_inner_paragraph(&mut self) -> Result<(), String> {
        while self.at_inner_text() {
            self.parse_inner_text()?;
        }
        Ok(())
    }

    fn parse_inner_text(&mut self) -> Result<(), String> {
        if self.current == "#OBTW" {
            self.parse_comment()
        } else if self.at_inner_text() {
            self.parse_body_item()
        } else {
            let msg = format!("unexpected {} inside a paragraph", Self::describe(&self.current));
            Err(self.syntax_err(&msg))
        }
    }

    fn parse_variable_define(&mut self) -> Result<(), String> {
        self.expect("#IHAZ")?;
        if !is_text(&self.current) {
            let msg = format!("expected a variable name but found {}", Self::describe(&self.current));
            return Err(self.syntax_err(&msg));
        }
        let name = self.current.clone();
        self.advance();
        self.expect("#ITIZ")?;
        self.push_frame();
        self.parse_text()?;
        let mut kids = self.pop_frame();
        self.expect("#MKAY")?;
        let value = match kids.pop() {
            Some(Node::Text(t)) => t,
            _ => String::new(),
        };
        self.push_node(Node::VarDef { name, value });
        Ok(())
    }

    fn parse_variable_use(&mut self) -> Result<(), String> {
        let (line, col) = self.positions[self.idx];
        self.expect("#GIMMEH VAR")?;
        if !is_text(&self.current) {
            let msg = format!("expected a variable name but found {}", Self::describe(&self.current));
            return Err(self.syntax_err(&msg));
        }
        let name = self.current.clone();
        self.advance();
        self.expect("#OIC")?;
        self.push_node(Node::VarUse { name, line, col });
        Ok(())
    }

    fn parse_bold(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH BOLD")?;
        self.push_frame();
        self.parse_define_opt()?;
        loop {
            if self.current == "#GIMMEH VAR" {
                self.parse_variable_use()?;
            } else if is_text(&self.current) {
                self.parse_word()?;
            } else {
                break;
            }
        }
        self.expect("#OIC")?;
        let kids = self.pop_frame();
        self.push_node(Node::Bold(kids));
        Ok(())
    }

    fn parse_list(&mut self) -> Result<(), String> {
        self.expect("#MAEK LIST")?;
        self.push_frame();
        self.parse_define_opt()?;
        self.parse_list_items()?;
        self.expect("#MKAY")?;
        let kids = self.pop_frame();
        self.push_node(Node::List(kids));
        Ok(())
    }

    fn parse_list_items(&mut self) -> Result<(), String> {
        // At least one item is required, then any mix of items and comments.
        self.parse_item()?;
        self.parse_inner_list()
    }

    fn parse_inner_list(&mut self) -> Result<(), String> {
        loop {
            if self.current == "#GIMMEH ITEM" {
                self.parse_item()?;
            } else if self.current == "#OBTW" {
                self.parse_comment()?;
            } else {
                return Ok(());
            }
        }
    }

    fn parse_link(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH LINX")?;
        if !is_text(&self.current) {
            let msg = format!("expected a link address but found {}", Self::describe(&self.current));
            return Err(self.syntax_err(&msg));
        }
        let addr = self.current.clone();
        self.advance();
        self.expect("#OIC")?;
        self.push_node(Node::Link(addr));
        Ok(())
    }

    fn parse_newline(&mut self) -> Result<(), String> {
        self.expect("#NEWLINE")?;
        self.push_node(Node::Newline);
        Ok(())
    }

    /// <text> ::= TEXT <more_text>: one or more words, joined into one Text node.
    fn parse_text(&mut self) -> Result<(), String> {
        if !is_text(&self.current) {
            let msg = format!("expected text but found {}", Self::describe(&self.current));
            return Err(self.syntax_err(&msg));
        }
        let mut words: Vec<String> = Vec::new();
        while is_text(&self.current) {
            words.push(self.current.clone());
            self.advance();
        }
        self.push_node(Node::Text(words.join(" ")));
        Ok(())
    }
}
