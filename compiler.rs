//Ryan Domathoti
// Ties the phases together: lex -> parse -> resolve scopes -> generate HTML.

use crate::codegen;
use crate::diagnostics::Reporter;


use crate::lexer::Lexer;
use crate::parser::Parser;

use crate::semantics;
use crate::traits::{Compiler, SyntaxAnalyzer};

pub struct LolCompiler {
    file_name: String,
    lexer: Lexer,
    parser: Parser,
    html: Option<String>,
}

impl LolCompiler {
    pub fn new(file_name: &str) -> Self {
        let rep = Reporter::new(file_name, "");
        LolCompiler {
            file_name: file_name.to_string(),
            lexer: Lexer::new("", rep.clone()),
            parser: Parser::new(rep),
            html: None,
        }
    }

    /// The generated HTML, available only after a successful `compile`.
    pub fn html(&self) -> Option<&str> {
        self.html.as_deref()
    }
}

impl Compiler for LolCompiler {
    fn compile(&mut self, source: &str) {
        let rep = Reporter::new(&self.file_name, source);
        self.lexer = Lexer::new(source, rep.clone());
        let (tokens, positions) = self.lexer.tokenize(); // lexical errors exit here
        self.parser = Parser::new(rep.clone());
        self.parser.load(tokens, positions);
        self.parse(); // syntax errors exit here
        let tree = match self.parser.take_tree() {
            Some(t) => t,
            None => return,
        };
        match semantics::resolve(&tree, &rep) {
            Ok(resolved) => self.html = Some(codegen::generate(&resolved)),
            Err(msg) => {
                eprintln!("{}", msg);
                std::process::exit(1);
            }
        }
    }

    fn next_token(&mut self) -> String {
        self.lexer.next_token()
    }

    fn parse(&mut self) {
        if let Err(msg) = self.parser.parse_lolcode() {
            eprintln!("{}", msg);
            std::process::exit(1);
        }
    }

    fn current_token(&self) -> String {
        self.parser.current().to_string()
    }

    fn set_current_token(&mut self, tok: String) {
        self.parser.set_current(tok);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::LexicalAnalyzer;

    fn compile_ok(src: &str) -> String {
        let mut c = LolCompiler::new("test.lol");
        c.compile(src);
        c.html().unwrap().to_string()
    }

    fn parser_for(src: &str) -> Parser {
        let rep = Reporter::new("test.lol", src);
        let mut lx = Lexer::new(src, rep.clone());
        let (t, p) = lx.tokenize();
        let mut ps = Parser::new(rep);
        ps.load(t, p);
        ps
    }

    #[test]
    fn tokens_are_normalized_and_case_insensitive() {
        let rep = Reporter::new("t.lol", "");
        let mut lx = Lexer::new("#hai #Maek\n  head Hello #KBYE", rep);
        let (toks, _) = lx.tokenize();
        assert_eq!(toks, vec!["#HAI", "#MAEK HEAD", "Hello", "#KBYE", ""]);
    }

    #[test]
    fn lookup_accepts_only_known_annotations() {
        let lx = Lexer::new("", Reporter::new("t.lol", ""));
        assert!(lx.lookup("#GIMMEH BOLD"));
        assert!(lx.lookup("#kbye"));
        assert!(!lx.lookup("#HEAD"));
        assert!(!lx.lookup("#GIMMEH"));
    }

    #[test]
    fn next_token_and_current_token() {
        let mut c = LolCompiler::new("t.lol");
        assert_eq!(c.next_token(), "");
        c.set_current_token("#HAI".to_string());
        assert_eq!(c.current_token(), "#HAI");
    }

    #[test]
    fn simple_document() {
        let html = compile_ok("#HAI #MAEK HEAD #GIMMEH TITLE Test 2 #OIC #MKAY This is test case 2. #KBYE");
        assert_eq!(
            html,
            "<html>\n    <head>\n        <title>Test 2</title>\n    </head>\n    This is test case 2.\n</html>\n"
        );
    }

    #[test]
    fn bold_and_punctuation_gluing() {
        let html = compile_ok("#HAI This is #GIMMEH BOLD test case 3 #OIC . #KBYE");
        assert!(html.contains("This is <b>test case 3</b>."));
    }

    #[test]
    fn static_scoping_shadows_then_restores() {
        let src = "#HAI #IHAZ x #ITIZ outer #MKAY \
                   #MAEK PARAGRAF #IHAZ x #ITIZ inner #MKAY #GIMMEH VAR x #OIC #MKAY \
                   #GIMMEH VAR x #OIC #KBYE";
        let html = compile_ok(src);
        assert!(html.contains("<p>\n        inner\n    </p>"));
        assert!(html.contains("    outer\n"));
    }

    #[test]
    fn list_link_and_newline() {
        let html = compile_ok(
            "#HAI #MAEK LIST #GIMMEH ITEM a #OIC #GIMMEH ITEM b #OIC #MKAY #GIMMEH LINX www.x.com #OIC #NEWLINE #KBYE",
        );
        assert!(html.contains("<li>a</li>"));
        assert!(html.contains("<li>b</li>"));
        assert!(html.contains("<a href=\"www.x.com\">www.x.com</a> <br>"));
    }

    #[test]
    fn syntax_errors_are_returned() {
        assert!(parser_for("#HAI #MKAY #KBYE").parse_lolcode().is_err());
        assert!(parser_for("hello #KBYE").parse_lolcode().is_err());
        assert!(parser_for("#HAI #MAEK PARAGRAF #MAEK PARAGRAF #MKAY #MKAY #KBYE").parse_lolcode().is_err());
        assert!(parser_for("#HAI #MAEK LIST #MKAY #KBYE").parse_lolcode().is_err());
        assert!(parser_for("#HAI #KBYE extra").parse_lolcode().is_err());
        assert!(parser_for("#HAI #KBYE").parse_lolcode().is_ok());
    }
}
