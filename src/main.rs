extern crate syn;

use std::env;
use std::fs;
use std::io::Read;

#[derive(Debug)]
struct File {
    functions: Vec<Function>,
}

#[derive(Debug)]
struct Function {
    body: Vec<Command>,
}

#[derive(Debug)]
enum Command {
    PrintLn(String),
}

fn parse_body(body: &syn::Block) -> Result<Vec<Command>, String> {
    let mut stmts = vec![];
    for stmt in &body.stmts {
        match stmt {
            syn::Stmt::Item(syn::Item::Macro(ref im)) => {
                let macro_name = im.mac.path.segments[0].ident.as_ref();
                if macro_name == "println" {
                    match im.mac.tts.clone().into_iter().next() {
                        Some(ref arg) => {
                            stmts.push(Command::PrintLn(arg.to_string().replace("\"", "")));
                        }
                        None => return Err("Expected argument in function".into()),
                    }
                } else {
                    return Err(format!("Unknown macro: {}", macro_name));
                }
            }
            _ => return Err("Unexpected statement in function".into()),
        }
    }
    Ok(stmts)
}

fn parse_fn(item: &syn::ItemFn) -> Result<Function, String> {
    Ok(Function {
        body: parse_body(&item.block)?,
    })
}

fn parse_file(file: &syn::File) -> Result<File, String> {
    let mut functions = vec![];
    for item in &file.items {
        match item {
            syn::Item::Fn(ref item_fn) => {
                functions.push(parse_fn(item_fn)?);
            }
            _ => return Err("Unexpected item in file".into()),
        }
    }
    Ok(File { functions })
}

fn compile_to_c(file: &File) -> String {
    let mut c_output = String::new();

    c_output += "#include <stdio.h>\n";
    c_output += "int main() {\n";

    for stmt in &file.functions[0].body {
        match stmt {
            Command::PrintLn(ref msg) => {
                c_output += &format!("puts(\"{}\");\n", msg);
            }
        }
    }

    c_output += "}\n";
    c_output
}

fn compile_to_asm(file: &File) -> String {
    let mut asm_code_output = String::new();
    let mut asm_data_output = String::new();

    asm_data_output += ".data\n";

    asm_code_output += ".text\n";
    asm_code_output += ".global main\n";
    asm_code_output += "main:\n";

    let mut temp_num = 0;
    for stmt in &file.functions[0].body {
        match stmt {
            Command::PrintLn(ref msg) => {
                asm_code_output += &format!("mov $str_{}, %edi\n", temp_num);
                asm_code_output += "call puts\n";

                asm_data_output += &format!("str_{}:\n", temp_num);
                asm_data_output += &format!(" .asciz \"{}\"", msg);
                temp_num += 1;
            }
        }
    }

    asm_code_output + &asm_data_output
}

fn main() {
    let mut args = env::args();
    args.next();

    match args.next() {
        Some(fname) => {
            let mut src = String::new();
            let mut file = fs::File::open(&fname).expect("Unable to open file");
            file.read_to_string(&mut src).expect("Unable to read file");

            let syntax = syn::parse_file(&src).expect("Unable to parse file");
            let result = parse_file(&syntax);

            //println!("{}", compile_to_c(&result.unwrap()));
            println!("{}", compile_to_asm(&result.unwrap()));
        }
        _ => {
            println!("Please supply the file to compile");
        }
    }
}
