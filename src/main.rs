use std::{env, fs, process};

use mini_c::{
    codegen::tac_code_gen::translate_program,
    interpreter::interpret,
    parser::program,
    semantic::type_check,
};

fn usage() -> ! {
    eprintln!("Usage: minic --check <file.minic>");
    eprintln!("       minic --run   <file.minic>");
    eprintln!("       minic --tac   <file.minic>");
    process::exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        usage();
    }

    let flag = &args[1];
    let path = &args[2];

    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading '{}': {}", path, e);
            process::exit(1);
        }
    };

    let unchecked = match program(&source) {
        Ok((_, prog)) => prog,
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            process::exit(1);
        }
    };

    let checked = match type_check(&unchecked) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Type error: {}", e);
            process::exit(1);
        }
    };

    match flag.as_str() {
        "--check" => {
            println!("'{}' is well-typed.", path);
        }
        "--run" => {
            if let Err(e) = interpret(&checked) {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
        "--tac" => {
            let tac = translate_program(checked);
            for instruction in tac {
                println!("{instruction}");
            }
        }
        _ => usage(),
    }
}
