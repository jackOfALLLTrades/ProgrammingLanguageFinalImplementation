//! Character-by-character lexical analyzer (Task 1).

use crate::diagnostics::Reporter;
use crate::traits::LexicalAnalyzer;

/// Token returned when the input is exhausted.
pub const EOF: &str = "";

/// Every legal annotation, in normalized (upper-case, single-space) form.
const ANNOTATIONS: [&str; 17] = [
    "#HAI", "#KBYE", "#OBTW", "#TLDR", "#MAEK HEAD", "#MAEK PARAGRAF", "#MAEK LIST", "#MKAY",
    "#GIMMEH TITLE", "#GIMMEH BOLD", "#GIMMEH ITEM", "#GIMMEH LINX", "#GIMMEH VAR", "#OIC",
    "#NEWLINE", "#IHAZ", "#ITIZ",
];

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    /// The token currently being built.
    lexeme: String,
    /// Where the most recent token started.
    tok_line: usize,
    tok_col: usize,
    reporter: Reporter,
}

impl Lexer {
    pub fn new(source: &str, reporter: Reporter) -> Self {
        // Drop a UTF-8 byte order mark if the editor added one.
        let clean = source.strip_prefix('\u{feff}').unwrap_or(source);
        Lexer {
            chars: clean.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            lexeme: String::new(),
            tok_line: 1,
            tok_col: 1,
            reporter,
        }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.get_char();
            } else {
                break;
            }
        }
    }

    /// Read consecutive letters into the current lexeme.
    fn take_letters(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_alphabetic() {
                let ch = self.get_char();
                self.add_char(ch);
            } else {
                break;
            }
        }
    }

    /// Report a lexical error and stop (no output file is written).
    fn lexical_error(&self, msg: &str) -> ! {
        let shown = self.lexeme.trim_end().chars().count();
        eprintln!("{}", self.reporter.render("Lexical error", self.tok_line, self.tok_col, shown, msg));
        std::process::exit(1);
    }

    /// Start state is '#': read the annotation name. #MAEK and #GIMMEH need a
    /// second word, which is folded into the same token so the parser only
    /// needs one token of lookahead.
    fn lex_annotation(&mut self) {
        let hash = self.get_char();
        self.add_char(hash);
        self.take_letters();
        let first = self.lexeme.to_uppercase();
        if first == "#MAEK" || first == "#GIMMEH" {
            self.skip_whitespace();
            self.lexeme.push(' ');
            self.take_letters();
        }
        let normalized = self.lexeme.to_uppercase();
        if !self.lookup(&normalized) {
            let msg = format!("unknown annotation '{}'", self.lexeme.trim_end());
            self.lexical_error(&msg);
        }
        self.lexeme = normalized;
    }

    /// Plain text: one word, ending at whitespace or at the next '#'.
    fn lex_text(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() || c == '#' {
                break;
            }
            let ch = self.get_char();
            self.add_char(ch);
        }
    }

    /// Produce the next token, or EOF ("") when the input is used up.
    pub fn next_token(&mut self) -> String {
        self.skip_whitespace();
        self.lexeme.clear();
        self.tok_line = self.line;
        self.tok_col = self.col;
        if self.at_end() {
            return EOF.to_string();
        }
        if self.peek() == Some('#') {
            self.lex_annotation();
        } else {
            self.lex_text();
        }
        self.lexeme.clone()
    }

    /// Position (line, col) where the last token returned started.
    pub fn last_position(&self) -> (usize, usize) {
        (self.tok_line, self.tok_col)
    }

    /// Lex the whole source up front. The final token is always EOF.
    pub fn tokenize(&mut self) -> (Vec<String>, Vec<(usize, usize)>) {
        let mut tokens = Vec::new();
        let mut positions = Vec::new();
        loop {
            let tok = self.next_token();
            positions.push(self.last_position());
            let done = tok == EOF;
            tokens.push(tok);
            if done {
                break;
            }
        }
        (tokens, positions)
    }
}

impl LexicalAnalyzer for Lexer {
    fn get_char(&mut self) -> char {
        if self.at_end() {
            eprintln!("internal error: lexer tried to read past the end of the input");
            std::process::exit(1);
        }
        let c = self.chars[self.pos];
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        c
    }

    fn add_char(&mut self, c: char) {
        self.lexeme.push(c);
    }

    fn lookup(&self, s: &str) -> bool {
        ANNOTATIONS.contains(&s.to_uppercase().as_str())
    }
}
