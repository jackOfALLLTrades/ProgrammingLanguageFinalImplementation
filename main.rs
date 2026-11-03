//! lolcompiler: translates a LOLCODE Markdown (.lol) file into HTML.
//! Usage: lolcompiler <input.lol>

mod ast;
mod codegen;
mod compiler;
mod diagnostics;
mod lexer;
mod parser;
mod semantics;
mod traits;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

use compiler::LolCompiler;
use traits::Compiler;

fn usage_error(msg: &str) -> ! {
    eprintln!("error: {}", msg);
    eprintln!("usage: lolcompiler <input.lol>");
    exit(1);
}

/// Ask the OS to open the file in the default browser.
fn open_in_browser(file: &Path) {
    // Skip when testing in bulk (set LOL_NO_BROWSER=1).
    if env::var_os("LOL_NO_BROWSER").is_some() {
        return;
    }
    let abs: PathBuf = if file.is_absolute() {
        file.to_path_buf()
    } else {
        env::current_dir().map(|d| d.join(file)).unwrap_or_else(|_| file.to_path_buf())
    };
    let result = if cfg!(target_os = "windows") {
        Command::new("cmd").arg("/C").arg("start").arg("").arg(&abs).status()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(&abs).status()
    } else {
        Command::new("xdg-open").arg(&abs).status()
    };
    if result.is_err() {
        eprintln!("warning: could not open a browser; open {} yourself", abs.display());
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        usage_error("expected exactly one argument, the input file");
    }
    let input = Path::new(&args[1]);
    let is_lol = input
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("lol"))
        .unwrap_or(false);
    if !is_lol {
        usage_error(&format!("'{}' does not have a .lol extension", args[1]));
    }
    let source = match fs::read_to_string(input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read '{}': {}", args[1], e);
            exit(1);
        }
    };

    let mut compiler = LolCompiler::new(&args[1]);
    compiler.compile(&source); // any error exits here, before a file is written

    let output = input.with_extension("html");
    let html = compiler.html().unwrap_or("");
    if let Err(e) = fs::write(&output, html) {
        eprintln!("error: cannot write '{}': {}", output.display(), e);
        exit(1);
    }
    println!("Compiled {} -> {}", input.display(), output.display());
    open_in_browser(&output);
}
