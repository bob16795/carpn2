use crate::{lexer, parser::*};
use std::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ProcData {
    full_name: String,
    args: u8,
    rets: bool,
    ret_kind: String,
}

#[derive(Clone, Debug)]
pub enum StackEntry {
    Type(String),
    Value(String, HashMap<String, StackEntry>),
    Struct(String, VisitableCtx),
    Field(String, HashMap<String, StackEntry>),
    SelfType,
}

#[derive(Clone, Debug)]
pub struct VisitableCtx {
    pub stack: Rc<RefCell<Vec<StackEntry>>>,
    pub vars: HashMap<String, StackEntry>,
    pub procs: HashMap<String, ProcData>,
    pub var_idx: usize,
    pub indent: Rc<RefCell<usize>>,
    pub inside: String,
    pub in_struct: Option<String>,
    pub in_proc: bool,
    pub h_file: String,
    pub cache: PathBuf,
    pub c_files: Vec<String>,
}

impl VisitableCtx {
    pub fn ind(&self) -> String {
        "\n".to_string() + &" ".repeat(*self.indent.borrow_mut() * 4)
    }
}

pub trait Visitable {
    fn header(&self, ctx: &mut VisitableCtx) -> String {
        "".to_string()
    }
    fn source(&self, ctx: &mut VisitableCtx) -> String {
        "".to_string()
    }
}

impl Visitable for Expression {
    fn source(&self, ctx: &mut VisitableCtx) -> String {
        match self {
            Expression::Ident(i) if i == "disc" => {
                _ = ctx.stack.borrow_mut().pop();
                "".to_string()
            }
            Expression::Ident(i) if i == "Self" => {
                ctx.stack.borrow_mut().push(StackEntry::SelfType);
                "".to_string()
            }
            Expression::Ident(i) if i == "swap" => {
                let Some(a) = ctx.stack.borrow_mut().pop() else {
                    todo!()
                };
                let Some(b) = ctx.stack.borrow_mut().pop() else {
                    todo!()
                };
                ctx.stack.borrow_mut().push(a);
                ctx.stack.borrow_mut().push(b);

                "".to_string()
            }
            Expression::Ident(i) if i == "copy" => {
                let Some(tmp) = ctx.stack.borrow_mut().pop() else {
                    todo!()
                };
                ctx.stack.borrow_mut().push(tmp.clone());
                ctx.stack.borrow_mut().push(tmp.clone());
                "".to_string()
            }
            Expression::Ident(i) => {
                if let Some(pushes) = ctx.vars.get(i) {
                    ctx.stack.borrow_mut().push(pushes.clone());
                    "".to_string()
                } else if let Some(proc) = ctx.procs.get(i) {
                    let mut args = Vec::new();

                    for a in 0..proc.args {
                        args.push(ctx.stack.borrow_mut().pop().unwrap());
                    }

                    let mut call = format!("{}(", proc.full_name);
                    let mut add = false;
                    while let Some(a) = args.pop() {
                        let StackEntry::Value(a, _) = a else { todo!() };
                        if add {
                            call += ",";
                        }
                        add = true;
                        call += &a;
                    }

                    call += ");";

                    call += &ctx.ind();

                    if proc.rets {
                        ctx.stack.borrow_mut().push(StackEntry::Value(
                            format!("anon_{}", ctx.var_idx),
                            HashMap::new(),
                        ));

                        call = format!("{} anon_{} = {}", proc.ret_kind, ctx.var_idx, call);

                        ctx.var_idx += 1;
                    }

                    call
                } else {
                    let top = ctx.stack.borrow_mut().pop();

                    if let Some(StackEntry::Type(k)) = top {
                        if ctx.in_proc {
                            let var_name = format!("var_{}", ctx.var_idx);
                            ctx.var_idx += 1;

                            let mut var = format!("{} {};", k, var_name);
                            var += &ctx.ind();

                            ctx.vars.insert(
                                i.clone(),
                                StackEntry::Value(format!("&{}", var_name), HashMap::new()),
                            );

                            return var;
                        } else {
                            let field_name = format!("field_{}", ctx.var_idx);
                            ctx.var_idx += 1;

                            ctx.vars.insert(
                                i.clone(),
                                StackEntry::Field(field_name.clone(), HashMap::new()),
                            );

                            let mut var = format!("{} {};", k, field_name);
                            var += &ctx.ind();

                            return var;
                        }
                    } else if let Some(StackEntry::Struct(name, sctx)) = top {
                        if ctx.in_proc {
                            let var_name = format!("var_{}", ctx.var_idx);
                            ctx.var_idx += 1;

                            let mut var = format!("{} {};", name, var_name);
                            var += &ctx.ind();

                            ctx.vars.insert(
                                i.clone(),
                                StackEntry::Value(format!("&{}", var_name), sctx.vars.clone()),
                            );

                            return var;
                        } else {
                            let field_name = format!("field_{}", ctx.var_idx);
                            ctx.var_idx += 1;

                            ctx.vars.insert(
                                i.clone(),
                                StackEntry::Field(field_name.clone(), sctx.vars.clone()),
                            );

                            let mut var = format!("{} {};", name, field_name);
                            var += &ctx.ind();

                            return var;
                        }
                    };

                    todo!("err: {}", i)
                }
            }
            Expression::Prop(p) => {
                let mut top = ctx.stack.borrow_mut().pop().unwrap();

                println!("{:?}", p);

                match &mut top {
                    StackEntry::Value(v, map) => match map.get(p) {
                        Some(StackEntry::Field(f, map)) => ctx
                            .stack
                            .borrow_mut()
                            .push(StackEntry::Value(format!("{}.{}", v, f), map.clone())),
                        s => todo!("{:?}", s),
                    },
                    StackEntry::Type(v) => ctx
                        .stack
                        .borrow_mut()
                        .push(StackEntry::Type(format!("{}_{}", v, p))),
                    StackEntry::Struct(_, v) => return Expression::Ident(p.clone()).source(v),
                    StackEntry::SelfType => return Expression::Ident(p.clone()).source(ctx),
                    s => todo!("{:?}", s),
                }
                "".to_string()
            }
            Expression::Op(ExprOp::GreaterThan) => {
                let StackEntry::Value(b, _) = ctx.stack.borrow_mut().pop().unwrap() else {
                    todo!()
                };
                let StackEntry::Value(a, _) = ctx.stack.borrow_mut().pop().unwrap() else {
                    todo!()
                };
                ctx.stack
                    .borrow_mut()
                    .push(StackEntry::Value(format!("{} > {}", a, b), HashMap::new()));
                "".to_string()
            }
            Expression::Op(ExprOp::LessThan) => {
                let StackEntry::Value(b, _) = ctx.stack.borrow_mut().pop().unwrap() else {
                    todo!()
                };
                let StackEntry::Value(a, _) = ctx.stack.borrow_mut().pop().unwrap() else {
                    todo!()
                };
                ctx.stack
                    .borrow_mut()
                    .push(StackEntry::Value(format!("{} < {}", a, b), HashMap::new()));
                "".to_string()
            }
            Expression::Op(ExprOp::Star) => {
                let top = ctx.stack.borrow_mut().pop().unwrap();
                match top {
                    StackEntry::Type(top) => {
                        let top = format!("{}*", top);
                        ctx.stack.borrow_mut().push(StackEntry::Type(top));
                        "".to_string()
                    }
                    _ => todo!(),
                }
            }
            Expression::Op(ExprOp::Dollar) => {
                let top = ctx.stack.borrow_mut().pop().unwrap();
                match top {
                    StackEntry::Type(top) => {
                        let top = format!("{}", top);
                        ctx.stack.borrow_mut().push(StackEntry::Type(top));
                        "".to_string()
                    }
                    _ => todo!(),
                }
            }
            Expression::Op(ExprOp::Equal) => {
                let StackEntry::Value(a, _) = ctx.stack.borrow_mut().pop().unwrap() else {
                    todo!()
                };
                let StackEntry::Value(b, _) = ctx.stack.borrow_mut().pop().unwrap() else {
                    todo!()
                };
                ctx.stack
                    .borrow_mut()
                    .push(StackEntry::Value(format!("{} == {}", a, b), HashMap::new()));
                "".to_string()
            }
            Expression::Op(ExprOp::Plus) => {
                let StackEntry::Value(b, _) = ctx.stack.borrow_mut().pop().unwrap() else {
                    todo!()
                };
                let StackEntry::Value(a, _) = ctx.stack.borrow_mut().pop().unwrap() else {
                    todo!()
                };
                ctx.stack
                    .borrow_mut()
                    .push(StackEntry::Value(format!("{} + {}", a, b), HashMap::new()));
                "".to_string()
            }
            Expression::Op(ExprOp::Assign) => {
                let StackEntry::Value(b, _) = ctx.stack.borrow_mut().pop().unwrap() else {
                    todo!()
                };
                let StackEntry::Value(a, _) = ctx.stack.borrow_mut().pop().unwrap() else {
                    todo!()
                };
                let mut assign = format!("*{} = {};", a, b);

                if assign.starts_with("*&") {
                    assign.remove(0);
                    assign.remove(0);
                }

                assign += &ctx.ind();

                assign
            }
            Expression::Op(ExprOp::Deref) => {
                let StackEntry::Value(a, _) = ctx.stack.borrow_mut().pop().unwrap() else {
                    todo!()
                };

                if a.starts_with('&') {
                    let mut tmp = a.clone();
                    tmp.remove(0);
                    ctx.stack
                        .borrow_mut()
                        .push(StackEntry::Value(format!("{}", tmp), HashMap::new()));
                } else {
                    ctx.stack
                        .borrow_mut()
                        .push(StackEntry::Value(format!("*{}", a), HashMap::new()));
                }
                "".to_string()
            }
            Expression::Op(ExprOp::Minus) => {
                let StackEntry::Value(b, _) = ctx.stack.borrow_mut().pop().unwrap() else {
                    todo!()
                };
                let StackEntry::Value(a, _) = ctx.stack.borrow_mut().pop().unwrap() else {
                    todo!()
                };
                ctx.stack
                    .borrow_mut()
                    .push(StackEntry::Value(format!("{} - {}", a, b), HashMap::new()));
                "".to_string()
            }
            Expression::Op(ExprOp::Name(n)) => {
                let Some(top) = ctx.stack.borrow_mut().pop() else {
                    todo!()
                };

                ctx.vars.insert(n.clone(), top);

                "".to_string()
            }
            Expression::Op(ExprOp::Tick) => {
                let StackEntry::Type(to) = ctx.stack.borrow_mut().pop().unwrap() else {
                    todo!()
                };
                let StackEntry::Value(top, _) = ctx.stack.borrow_mut().pop().unwrap() else {
                    todo!()
                };
                ctx.stack.borrow_mut().push(StackEntry::Value(
                    format!("({})({})", to, top),
                    HashMap::new(),
                ));
                "".to_string()
            }
            Expression::String(i) => {
                ctx.stack
                    .borrow_mut()
                    .push(StackEntry::Value(format!("\"{}\"", i), HashMap::new()));
                "".to_string()
            }
            Expression::Int(i) => {
                ctx.stack
                    .borrow_mut()
                    .push(StackEntry::Value(i.to_string(), HashMap::new()));
                "".to_string()
            }
            Expression::Return => {
                let StackEntry::Value(result, _) = ctx.stack.borrow_mut().pop().unwrap() else {
                    todo!()
                };

                format!("return {};", result)
            }
            Expression::If(expr) => {
                let StackEntry::Value(cond, _) = ctx.stack.borrow_mut().pop().unwrap() else {
                    todo!()
                };
                let mut result = format!("if ({}) {{", cond);
                *ctx.indent.borrow_mut() += 1;
                result += &ctx.ind();

                for b in &expr.body {
                    result += &b.source(ctx);
                }

                *ctx.indent.borrow_mut() -= 1;
                result += &ctx.ind();
                result += &format!("}}");
                result += &ctx.ind();

                result
            }
            t => todo!("{:?}", t),
        }
    }
}

impl Visitable for Prototype {
    fn header(&self, ctx: &mut VisitableCtx) -> String {
        let mut prefix = "".to_string();
        let mut result = "".to_string();
        if let Some(res) = &self.result {
            for r in res {
                prefix += &r.source(ctx);
            }
            if ctx.stack.borrow_mut().len() != 1 {
                todo!("{} {:?}", self.name, ctx.stack)
            }
            for v in ctx.stack.borrow_mut().iter() {
                let StackEntry::Type(v) = v else { todo!() };
                result += &v;
            }

            ctx.stack.borrow_mut().clear();
        } else {
            result += "void";
        }

        let ret_kind = result.clone();

        result += " ";
        result += &ctx.inside;
        result += &self.name;

        result += "(";
        for r in &self.args {
            prefix += &r.source(ctx);
        }

        ctx.procs.insert(
            self.name.clone(),
            ProcData {
                full_name: ctx.inside.clone() + &self.name.clone(),
                args: ctx.stack.borrow_mut().len() as u8,
                rets: self.result != None,
                ret_kind,
            },
        );

        let mut new_stack = Vec::new();

        let mut add = false;
        for (i, c) in ctx.stack.borrow_mut().iter().enumerate() {
            let StackEntry::Type(c) = c else { todo!() };
            if add {
                result += ",";
            }

            let arg = format!("arg_{}", i);
            add = true;
            result += c;
            result += " ";
            result += &arg;
            new_stack.push(StackEntry::Value(arg, HashMap::new()));
        }

        *ctx.stack.borrow_mut() = new_stack;

        result += ")";

        prefix + &result
    }
}

impl Visitable for Proc {
    fn source(&self, ctx: &mut VisitableCtx) -> String {
        match &self.body {
            ProcBody::Impl(body) => {
                let mut result = self.def.header(ctx);
                result += " {";
                *ctx.indent.borrow_mut() += 1;

                result += &ctx.ind();

                ctx.in_proc = true;
                for b in body {
                    result += &b.source(ctx);
                }
                ctx.in_proc = false;

                let mut stk = ctx.stack.borrow_mut();

                if let Some(StackEntry::Value(ret, _)) = stk.pop() {
                    result += &format!("return {};", ret);

                    if stk.len() != 0 {
                        todo!("{}", result);
                    }
                }

                *ctx.indent.borrow_mut() -= 1;

                result += &ctx.ind();

                result += "}";
                result += &ctx.ind();

                result
            }
            _ => "".to_string(),
            // _ => format!("// extern {}\n", self.def.name),
        }
    }
}

impl Visitable for Struct {
    fn header(&self, ctx: &mut VisitableCtx) -> String {
        let mut new_ctx = ctx.clone();
        new_ctx.inside += &self.name;
        new_ctx.inside += "_";

        if new_ctx.in_struct != None {
            new_ctx.in_struct = Some(new_ctx.in_struct.clone().unwrap() + &"_" + &self.name);
        } else {
            new_ctx.in_struct = Some(self.name.clone());
        }

        let mut result = "".to_string();

        for s in &self.structs {
            result += &s.header(&mut new_ctx);
        }

        result += &format!("typedef struct {{");
        *ctx.indent.borrow_mut() += 1;
        result += &ctx.ind();

        for s in &self.body {
            result += &s.header(&mut new_ctx);
        }

        result = result.trim_end_matches(' ').to_string();
        *ctx.indent.borrow_mut() -= 1;

        result += &format!("}} {};", new_ctx.in_struct.clone().unwrap());
        result += &ctx.ind();
        result += &ctx.ind();

        for p in &self.procs {
            result += &p.def.header(&mut new_ctx);
            result += ";";
            result += &ctx.ind();
            new_ctx.stack.borrow_mut().clear();
        }
        result += &ctx.ind();

        ctx.vars.insert(
            self.name.clone(),
            StackEntry::Struct(new_ctx.clone().in_struct.unwrap(), new_ctx.clone()),
        );

        result
    }

    fn source(&self, ctx: &mut VisitableCtx) -> String {
        let Some(StackEntry::Struct(_, ref mut new_ctx)) = ctx.vars.get_mut(&self.name) else {
            todo!();
        };

        let mut result = "".to_string();

        for p in &self.procs {
            result += &p.source(new_ctx);
            result += &new_ctx.ind();
        }

        println!("{:?}", new_ctx);

        result
    }
}

impl Visitable for Include {
    fn header(&self, ctx: &mut VisitableCtx) -> String {
        if self.is_c {
            return if self.file.starts_with("<") {
                format!("#include {}", self.file)
            } else {
                format!("#include \"{}\"", self.file)
            };
        }

        let file = PathBuf::from(self.file.clone());

        let old = ctx.h_file.clone();

        let lex = lexer::Lexer::new(file.clone()).unwrap();
        let tmp = CarpnFile::parse(&mut lex.peekable()).unwrap();
        let mut h_file = ctx.cache.clone();
        let mut c_file = ctx.cache.clone();

        c_file.push(file.clone());
        h_file.push(file.clone());

        c_file.set_extension("c");
        h_file.set_extension("h");

        ctx.h_file = format!("{}", h_file.display());

        let h_conts = tmp.header(ctx);
        let c_conts = tmp.source(ctx);

        let mut file = File::create(c_file.clone()).unwrap();
        file.write_all(c_conts.as_str().as_bytes()).unwrap();

        let mut file = File::create(h_file.clone()).unwrap();
        file.write_all(h_conts.as_str().as_bytes()).unwrap();

        ctx.h_file = old;
        ctx.c_files.push(format!("{}", c_file.display()));

        format!("#include \"{}\"", h_file.display())
    }
}

impl Visitable for CarpnFile {
    fn header(&self, ctx: &mut VisitableCtx) -> String {
        let mut result = "".to_string();

        let header_name = format!("_{}", ctx.h_file.to_uppercase().replace(".", "_"));

        result += &format!("#ifndef {}\n", header_name);
        result += &format!("#define {}\n", header_name);

        for i in &self.includes {
            result += &i.header(ctx);
            result += &ctx.ind();
        }

        for s in &self.structs {
            result += &s.header(ctx);
        }

        for p in &self.procs {
            if p.body == ProcBody::Extern {
                result += "//";
            }
            result += &p.def.header(ctx);
            result += ";";
            result += &ctx.ind();
            ctx.stack.borrow_mut().clear();
        }

        result += &format!("#endif\n");

        result
    }

    fn source(&self, ctx: &mut VisitableCtx) -> String {
        let mut result = "".to_string();

        result += &format!("#include \"{}\"\n", ctx.h_file);

        for s in &self.structs {
            result += &s.source(ctx);
        }

        for p in &self.procs {
            result += &p.source(ctx);
        }

        result
    }
}
