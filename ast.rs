
//Ryan Domathoti
//! Abstract syntax tree produced by the parser. 
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// `#HAI ... #KBYE`
    Html(Vec<Node>),
    /// `#MAEK HEAD ... #MKAY`
    Head(Vec<Node>),
    /// `#GIMMEH TITLE text #OIC`
    Title(String),
    /// `#OBTW text #TLDR`
    Comment(String),
    /// `#MAEK PARAGRAF ... #MKAY`
    Paragraph(Vec<Node>),
    /// `#GIMMEH BOLD ... #OIC`
    Bold(Vec<Node>),
    /// `#MAEK LIST ... #MKAY`
    List(Vec<Node>),
    /// `#GIMMEH ITEM ... #OIC`
    Item(Vec<Node>),
    /// `#GIMMEH LINX address #OIC`
    Link(String),
    /// `#NEWLINE`
    Newline,
    /// Plain text (one or more words joined by single spaces).
    Text(String),
    /// `#IHAZ name #ITIZ value #MKAY`
    VarDef { name: String, value: String },
    /// `#GIMMEH VAR name #OIC` (line/col kept for semantic error messages)
    VarUse { name: String, line: usize, col: usize },
}
