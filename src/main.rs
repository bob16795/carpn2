mod cli;
mod error;
mod lexer;
mod parser;
mod visit;

use cli::*;
use error::*;
use lexer::*;
use parser::*;
use visit::*;

use clap::{Command, Parser};

use dirs::cache_dir;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::process;
use std::rc::Rc;

fn main() -> Result<(), String> {
    let args = Args::parse();

    match args {
        Args::C(c_args) => {
            for file in c_args.input {
                let lex = Lexer::new(file.clone())?;

                let tmp = CarpnFile::parse(&mut lex.peekable()).unwrap();

                let mut cache = cache_dir().unwrap();
                cache.push("carpn2");

                create_dir_all(cache.clone()).unwrap();

                let mut h_file = cache.clone();
                let mut c_file = cache.clone();

                c_file.push(file.clone());
                h_file.push(file.clone());

                c_file.set_extension("c");
                h_file.set_extension("h");

                let mut ctx = VisitableCtx {
                    stack: Rc::new(RefCell::new(Vec::new())),
                    vars: HashMap::new(),
                    procs: HashMap::new(),
                    inside: "".to_string(),
                    in_struct: None,
                    var_idx: 0,
                    indent: Rc::new(RefCell::new(0)),
                    in_proc: false,
                    h_file: format!("{}", h_file.display()),
                    cache,
                    c_files: vec![format!("{}", c_file.display())],
                };

                ctx.vars
                    .insert("void".to_string(), StackEntry::Type("void".to_string()));
                ctx.vars.insert(
                    "null".to_string(),
                    StackEntry::Value("(void*)(0)".to_string(), HashMap::new()),
                );
                ctx.vars
                    .insert("i32".to_string(), StackEntry::Type("int".to_string()));
                ctx.vars
                    .insert("i8".to_string(), StackEntry::Type("char".to_string()));

                println!("{:?}", tmp);
                println!("==========");

                let h_conts = tmp.header(&mut ctx);
                let c_conts = tmp.source(&mut ctx);

                let mut file = File::create(c_file.clone()).unwrap();
                file.write_all(c_conts.as_str().as_bytes()).unwrap();

                let mut file = File::create(h_file.clone()).unwrap();
                file.write_all(h_conts.as_str().as_bytes()).unwrap();

                process::Command::new(c_args.cc.as_str())
                    .args(ctx.c_files)
                    .output()
                    .unwrap();
            }

            Ok(())
        }
    }
}
