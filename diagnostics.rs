//Ryan Domathoti
//! Compiler-style error messages (kind, file:line:col, source line, caret).

/// Holds the file name and source text so any phase can print a nice error.
#[derive(Clone)]
pub struct Reporter {
    file: String,
    source: String,
}

impl Reporter {
    pub fn new(file: &str, source: &str) -> Self {
        Reporter { file: file.to_string(), source: source.to_string() }
    }

    /// Build the message text. `len` is how many characters the caret underlines.
    pub fn render(&self, kind: &str, line: usize, col: usize, len: usize, msg: &str) -> String {
        let src_line = self.source.lines().nth(line.saturating_sub(1)).unwrap_or("");
        let num = line.to_string();
        let pad = " ".repeat(num.len());
        // Copy tabs from the source line so the caret lines up under the token.
        let lead: String = src_line
            .chars()
            .take(col.saturating_sub(1))
            .map(|c| if c == '\t' { '\t' } else { ' ' })
            .collect();
        format!(
            "{kind}: {msg}\n{pad}--> {file}:{line}:{col}\n{pad} |\n{num} | {src}\n{pad} | {lead}{carets}",
            kind = kind,
            msg = msg,
            pad = pad,
            file = self.file,
            line = line,
            col = col,
            num = num,
            src = src_line.trim_end(),
            lead = lead,
            carets = "^".repeat(len.max(1)),
        )
    }
}
