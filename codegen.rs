//Ryan Domathoti
//! HTML generation (Task 4): turns the resolved tree into HTML text.

use crate::ast::Node;

fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push_str("    ");
    }
}

/// Text that starts with punctuation sticks to the previous piece ("Josh." not "Josh .").
fn sticks_left(t: &str) -> bool {
    t.starts_with(|c| matches!(c, '.' | ',' | '!' | '?' | ':'))
}

/// HTML for nodes that flow inside a line of text, plus whether to glue it to the previous piece.
fn inline_piece(n: &Node) -> Option<(String, bool)> {
    match n {
        Node::Text(t) => Some((escape(t), sticks_left(t))),
        Node::Bold(k) => Some((format!("<b>{}</b>", inline(k)), false)),
        Node::Link(a) => {
            let href = escape(a).replace('"', "&quot;");
            Some((format!("<a href=\"{}\">{}</a>", href, escape(a)), false))
        }
        Node::Newline => Some(("<br>".to_string(), false)),
        Node::Comment(c) => Some((format!("<!-- {} -->", escape(c)), false)),
        _ => None,
    }
}

fn append(line: &mut String, piece: &str, glue: bool) {
    if !line.is_empty() && !glue {
        line.push(' ');
    }
    line.push_str(piece);
}

/// Render a run of inline nodes as one string (used for bold and list items).
fn inline(nodes: &[Node]) -> String {
    let mut line = String::new();
    for n in nodes {
        if let Some((piece, glue)) = inline_piece(n) {
            append(&mut line, &piece, glue);
        }
    }
    line
}

fn flush(line: &mut String, depth: usize, out: &mut String) {
    if !line.is_empty() {
        indent(out, depth);
        out.push_str(line);
        out.push('\n');
        line.clear();
    }
}

/// Emit the children of a block: inline pieces collect on a line, block elements get their own lines.
fn emit_nodes(nodes: &[Node], depth: usize, out: &mut String) {
    let mut line = String::new();
    for n in nodes {
        match n {
            Node::Comment(c) => {
                flush(&mut line, depth, out);
                indent(out, depth);
                out.push_str(&format!("<!-- {} -->\n", escape(c)));
            }
            Node::Paragraph(k) => {
                flush(&mut line, depth, out);
                indent(out, depth);
                out.push_str("<p>\n");
                emit_nodes(k, depth + 1, out);
                indent(out, depth);
                out.push_str("</p>\n");
            }
            Node::List(items) => {
                flush(&mut line, depth, out);
                indent(out, depth);
                out.push_str("<ul>\n");
                for item in items {
                    indent(out, depth + 1);
                    match item {
                        Node::Item(k) => out.push_str(&format!("<li>{}</li>\n", inline(k))),
                        Node::Comment(c) => out.push_str(&format!("<!-- {} -->\n", escape(c))),
                        _ => {}
                    }
                }
                indent(out, depth);
                out.push_str("</ul>\n");
            }
            Node::Head(k) => {
                flush(&mut line, depth, out);
                indent(out, depth);
                out.push_str("<head>\n");
                for h in k {
                    indent(out, depth + 1);
                    match h {
                        Node::Title(t) => out.push_str(&format!("<title>{}</title>\n", escape(t))),
                        Node::Comment(c) => out.push_str(&format!("<!-- {} -->\n", escape(c))),
                        _ => {}
                    }
                }
                indent(out, depth);
                out.push_str("</head>\n");
            }
            other => {
                if let Some((piece, glue)) = inline_piece(other) {
                    append(&mut line, &piece, glue);
                    // A <br> ends the current line of output.
                    if matches!(other, Node::Newline) {
                        flush(&mut line, depth, out);
                    }
                }
            }
        }
    }
    flush(&mut line, depth, out);
}

/// Produce the full HTML document for a resolved tree.
pub fn generate(root: &Node) -> String {
    let mut out = String::new();
    out.push_str("<html>\n");
    if let Node::Html(k) = root {
        emit_nodes(k, 1, &mut out);
    }
    out.push_str("</html>\n");
    out
}
