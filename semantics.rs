//Ryan Domathoti
// Static-scope variable resolution (Task 3). Walks the tree once, keeping a
// stack of scopes (one per block). A definition is visible from the point it
// appears to the end of its block; inner blocks may shadow outer ones.
// Every use is replaced by the variable's value, so code generation never
// sees a VarUse or VarDef.

use std::collections::HashMap;

use crate::ast::Node;
use crate::diagnostics::Reporter;

type Scopes = Vec<HashMap<String, String>>;

pub fn resolve(root: &Node, reporter: &Reporter) -> Result<Node, String> {
    let mut scopes: Scopes = Vec::new();
    resolve_node(root, &mut scopes, reporter)
}

fn resolve_block(kids: &[Node], scopes: &mut Scopes, rep: &Reporter) -> Result<Vec<Node>, String> {
    scopes.push(HashMap::new());
    let mut out = Vec::new();
    for kid in kids {
        match kid {
            Node::VarDef { name, value } => {
                if let Some(top) = scopes.last_mut() {
                    top.insert(name.clone(), value.clone());
                }
            }
            other => out.push(resolve_node(other, scopes, rep)?),
        }
    }
    scopes.pop();
    Ok(out)
}

fn resolve_node(node: &Node, scopes: &mut Scopes, rep: &Reporter) -> Result<Node, String> {
    Ok(match node {
        Node::Html(k) => Node::Html(resolve_block(k, scopes, rep)?),
        Node::Head(k) => Node::Head(resolve_block(k, scopes, rep)?),
        Node::Paragraph(k) => Node::Paragraph(resolve_block(k, scopes, rep)?),
        Node::Bold(k) => Node::Bold(resolve_block(k, scopes, rep)?),
        Node::List(k) => Node::List(resolve_block(k, scopes, rep)?),
        Node::Item(k) => Node::Item(resolve_block(k, scopes, rep)?),
        Node::VarUse { name, line, col } => {
            // Search from the innermost scope outward.
            match scopes.iter().rev().find_map(|s| s.get(name)) {
                Some(value) => Node::Text(value.clone()),
                None => {
                    let msg = format!("variable '{}' is used but was never defined in an enclosing scope", name);
                    // "#GIMMEH VAR" is 11 characters wide.
                    return Err(rep.render("Static semantic error", *line, *col, 11, &msg));
                }
            }
        }
        other => other.clone(),
    })
}
