use crate::{BlockKind, Lexer, Operation, ParseError, Token};
use std::iter::Peekable;

pub trait Parsable {
    fn parse(lex: &mut Peekable<Lexer>) -> Result<Self, ParseError>
    where
        Self: Sized;
}

#[derive(Debug, PartialEq)]
pub struct IfExpression {
    pub body: Vec<Expression>,
}

impl Parsable for IfExpression {
    fn parse(lex: &mut Peekable<Lexer>) -> Result<Self, ParseError> {
        if lex.peek() != Some(&Token::If) {
            return Err(ParseError::MissingBody);
        }

        _ = lex.next();

        if lex.peek() != Some(&Token::BlockOpen(BlockKind::Curly)) {
            return Err(ParseError::MissingBody);
        }

        _ = lex.next();

        let mut body = Vec::new();

        while let Ok(expr) = Expression::parse(lex) {
            body.push(expr);
        }

        println!("{:?}", body);

        if lex.peek() != Some(&Token::BlockClose(BlockKind::Curly)) {
            return Err(ParseError::MissingBody);
        }

        _ = lex.next();

        Ok(IfExpression { body })
    }
}

#[derive(Debug, PartialEq)]
pub enum ExprOp {
    Name(String),
    Equal,
    Assign,
    Star,
    Dollar,
    Minus,
    Plus,
    Tick,
    LessThan,
    GreaterThan,
    Deref,
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Int(i64),
    Float(f64),
    Ident(String),
    Op(ExprOp),
    If(IfExpression),
    String(String),
    Prop(String),
    Return,
}

impl Parsable for Expression {
    fn parse(lex: &mut Peekable<Lexer>) -> Result<Self, ParseError> {
        let mut tmp = lex.clone();
        let a = tmp.next();
        let b = tmp.next();
        match (a, b) {
            (Some(Token::Op(Operation::Dot)), Some(Token::Ident(name))) => {
                _ = lex.next();
                _ = lex.next();
                Ok(Expression::Prop(name))
            }
            (Some(Token::Op(Operation::Equal)), Some(Token::Op(Operation::Gt))) => {
                let Some(Token::Ident(name)) = tmp.next() else {
                    todo!();
                };

                _ = lex.next();
                _ = lex.next();
                _ = lex.next();
                Ok(Expression::Op(ExprOp::Name(name)))
            }
            (Some(Token::Op(Operation::Equal)), Some(Token::Op(Operation::Equal))) => {
                _ = lex.next();
                _ = lex.next();
                Ok(Expression::Op(ExprOp::Equal))
            }
            (Some(Token::Op(Operation::Equal)), _) => {
                _ = lex.next();
                Ok(Expression::Op(ExprOp::Assign))
            }
            (Some(Token::Op(Operation::Star)), _) => {
                _ = lex.next();
                Ok(Expression::Op(ExprOp::Star))
            }
            (Some(Token::Op(Operation::Dollar)), _) => {
                _ = lex.next();
                Ok(Expression::Op(ExprOp::Dollar))
            }
            (Some(Token::Op(Operation::Tick)), _) => {
                _ = lex.next();
                Ok(Expression::Op(ExprOp::Tick))
            }
            (Some(Token::Op(Operation::Gt)), _) => {
                _ = lex.next();
                Ok(Expression::Op(ExprOp::GreaterThan))
            }
            (Some(Token::Op(Operation::Plus)), _) => {
                _ = lex.next();
                Ok(Expression::Op(ExprOp::Plus))
            }
            (Some(Token::Op(Operation::Minus)), _) => {
                _ = lex.next();
                Ok(Expression::Op(ExprOp::Minus))
            }
            (Some(Token::Op(Operation::At)), _) => {
                _ = lex.next();
                Ok(Expression::Op(ExprOp::Deref))
            }
            (Some(Token::Op(Operation::Lt)), _) => {
                _ = lex.next();
                Ok(Expression::Op(ExprOp::LessThan))
            }
            (Some(Token::Ret), _) => {
                _ = lex.next();
                Ok(Expression::Return)
            }
            (Some(Token::Int(i)), _) => {
                _ = lex.next();
                Ok(Expression::Int(i))
            }
            (Some(Token::Float(f)), _) => {
                _ = lex.next();
                Ok(Expression::Float(f))
            }
            (Some(Token::Ident(s)), _) => {
                _ = lex.next();
                Ok(Expression::Ident(s.to_string()))
            }
            (Some(Token::String(s)), _) => {
                _ = lex.next();
                Ok(Expression::String(s))
            }
            (Some(Token::If), _) => {
                let expr = IfExpression::parse(lex)?;
                Ok(Expression::If(expr))
            }
            _ => Err(ParseError::InvalidExpression),
        }
    }
}

#[derive(Debug)]
pub struct Prototype {
    pub name: String,
    pub args: Vec<Expression>,
    pub result: Option<Vec<Expression>>,
}

impl Parsable for Prototype {
    fn parse(lex: &mut Peekable<Lexer>) -> Result<Self, ParseError> {
        let Some(Token::Ident(name)) = lex.next() else {
            return Err(ParseError::PrototypeMissingName);
        };

        let mut args = Vec::new();
        let mut result = None;

        while let Ok(expr) = Expression::parse(lex) {
            args.push(expr);
        }

        if lex.peek() == Some(&Token::Op(Operation::Colon)) {
            let mut tmp_result = Vec::new();
            _ = lex.next();

            while let Ok(expr) = Expression::parse(lex) {
                tmp_result.push(expr);
            }

            result = Some(tmp_result);
        }

        Ok(Prototype { name, args, result })
    }
}

#[derive(Debug, PartialEq)]
pub enum ProcBody {
    Extern,
    Impl(Vec<Expression>),
}

#[derive(Debug)]
pub struct Proc {
    pub def: Prototype,
    pub body: ProcBody,
}

impl Parsable for Proc {
    fn parse(lex: &mut Peekable<Lexer>) -> Result<Self, ParseError> {
        let Some(first) = lex.next() else {
            return Err(ParseError::ParserEOF);
        };

        let def = Prototype::parse(lex)?;

        let mut body = ProcBody::Extern;

        match first {
            Token::Proc => {
                if lex.peek() != Some(&Token::BlockOpen(BlockKind::Curly)) {
                    return Err(ParseError::MissingBody);
                }

                _ = lex.next();

                let mut body_conts = Vec::new();

                while let Ok(expr) = Expression::parse(lex) {
                    body_conts.push(expr);
                }

                body = ProcBody::Impl(body_conts);

                if lex.peek() != Some(&Token::BlockClose(BlockKind::Curly)) {
                    print!("{:?}", body);

                    return Err(ParseError::MissingCloseCurly);
                }

                _ = lex.next();
            }
            Token::Extern => {}
            _ => {
                return Err(ParseError::Unreachable);
            }
        }

        Ok(Proc { def, body })
    }
}

#[derive(Debug)]
pub struct Struct {
    pub name: String,
    pub procs: Vec<Proc>,
    pub structs: Vec<Struct>,
    pub body: Vec<Expression>,
}

impl Parsable for Struct {
    fn parse(lex: &mut Peekable<Lexer>) -> Result<Self, ParseError> {
        if lex.next() != Some(Token::Struct) {
            return Err(ParseError::MissingBody);
        }

        let Some(Token::Ident(name)) = lex.next() else {
            return Err(ParseError::MissingStructName);
        };

        let mut procs = Vec::new();
        let mut structs = Vec::new();
        let mut body = Vec::new();

        if lex.peek() != Some(&Token::BlockOpen(BlockKind::Curly)) {
            return Err(ParseError::MissingBody);
        }

        lex.next();

        loop {
            let Some(first) = lex.peek() else {
                return Err(ParseError::MissingCloseCurly);
            };

            match first {
                Token::BlockClose(BlockKind::Curly) => {
                    break;
                }

                Token::Proc | Token::Extern => {
                    let p = Proc::parse(lex)?;

                    procs.push(p);
                }
                Token::Struct => {
                    let s = Struct::parse(lex)?;

                    structs.push(s);
                }
                _ => {
                    let t = Expression::parse(lex)?;

                    body.push(t);
                }
            }
        }

        if lex.peek() != Some(&Token::BlockClose(BlockKind::Curly)) {
            return Err(ParseError::MissingBody);
        }

        lex.next();

        Ok(Struct {
            name,
            structs,
            procs,
            body,
        })
    }
}

#[derive(Debug)]
pub struct Include {
    pub file: String,
    pub is_c: bool,
}

impl Parsable for Include {
    fn parse(lex: &mut Peekable<Lexer>) -> Result<Self, ParseError> {
        let mut tmp = lex.clone();
        let first = tmp.next();
        let Some(Token::String(file)) = tmp.next() else {
            todo!("{:?}", lex.peek())
        };
        match &first {
            Some(Token::CInclude) => {
                _ = lex.next();
                _ = lex.next();

                return Ok(Include {
                    file: file.to_string(),
                    is_c: true,
                });
            }
            Some(Token::Include) => {
                _ = lex.next();
                _ = lex.next();

                return Ok(Include {
                    file: file.to_string(),
                    is_c: false,
                });
            }
            _ => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct CarpnFile {
    pub includes: Vec<Include>,
    pub procs: Vec<Proc>,
    pub structs: Vec<Struct>,
}

impl Parsable for CarpnFile {
    fn parse(lex: &mut Peekable<Lexer>) -> Result<Self, ParseError> {
        let mut procs = Vec::new();
        let mut structs = Vec::new();
        let mut includes = Vec::new();

        loop {
            let Some(first) = lex.peek() else {
                break;
            };

            match first {
                Token::CInclude | Token::Include => {
                    let i = Include::parse(lex)?;

                    includes.push(i);
                }
                Token::Proc | Token::Extern => {
                    let p = Proc::parse(lex)?;

                    procs.push(p);
                }
                Token::Struct => {
                    let s = Struct::parse(lex)?;

                    structs.push(s);
                }

                _ => return Err(ParseError::ParserEOF),
            }
        }

        Ok(CarpnFile {
            includes,
            procs,
            structs,
        })
    }
}
