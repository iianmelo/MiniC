//! Imprime a AST que o parser produz, ainda sem tipos.
//!
//! Não faz parte da linguagem. Serve para olhar a árvore de um arquivo:
//!
//! ```text
//! cargo run --quiet --example dump_ast -- tests/fixtures/tac_simple.minic
//! ```

use std::{env, fs, process};

use mini_c::parser::program;

fn main() {
    let mut args = env::args().skip(1);
    let Some(path) = args.next() else {
        eprintln!("uso: dump_ast <arquivo.minic>");
        process::exit(1);
    };

    let source = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("erro ao ler '{path}': {e}");
            process::exit(1);
        }
    };

    match program(&source) {
        Ok((rest, prog)) => {
            println!("{prog:#?}");
            if !rest.trim().is_empty() {
                eprintln!("aviso: o parser parou antes do fim do arquivo");
            }
        }
        Err(e) => {
            eprintln!("erro de parse: {e:?}");
            process::exit(1);
        }
    }
}
