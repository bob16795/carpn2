use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Clone)]
pub enum Operation {
    Dollar,
    Colon,
    Minus,
    Equal,
    Plus,
    Star,
    Tick,
    Dot,
    Lt,
    Gt,
    At,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BlockKind {
    Curly,
    Bracket,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    CInclude,
    Include,
    Extern,
    Struct,
    Proc,
    Def,
    Ret,
    If,
    BlockOpen(BlockKind),
    BlockClose(BlockKind),
    Op(Operation),
    Int(i64),
    Float(f64),
    String(String),

    Ident(String),
}

impl Token {
    pub fn new(base: &String) -> Result<Self, String> {
        match base.as_str() {
            "extern" => Ok(Self::Extern),
            "struct" => Ok(Self::Struct),
            "cinc" => Ok(Self::CInclude),
            "proc" => Ok(Self::Proc),
            "inc" => Ok(Self::Include),
            "def" => Ok(Self::Def),
            "ret" => Ok(Self::Ret),
            "if" => Ok(Self::If),
            "$" => Ok(Self::Op(Operation::Dollar)),
            ":" => Ok(Self::Op(Operation::Colon)),
            "=" => Ok(Self::Op(Operation::Equal)),
            "-" => Ok(Self::Op(Operation::Minus)),
            "*" => Ok(Self::Op(Operation::Star)),
            "'" => Ok(Self::Op(Operation::Tick)),
            "+" => Ok(Self::Op(Operation::Plus)),
            "." => Ok(Self::Op(Operation::Dot)),
            "<" => Ok(Self::Op(Operation::Lt)),
            ">" => Ok(Self::Op(Operation::Gt)),
            "@" => Ok(Self::Op(Operation::At)),
            s => {
                if s.starts_with('"') && s.ends_with('"') {
                    let mut new = s.to_string();
                    new.remove(new.len() - 1);
                    new.remove(0);
                    return Ok(Self::String(new));
                } else {
                    let dot = s.chars().filter(|c| *c == '.').count() == 1;
                    let too_many = s.chars().filter(|c| *c == '.').count() > 1;
                    let number = s.chars().filter(|c| *c != '.').all(|c| c.is_numeric());
                    let chars = s.chars().all(|c| c.is_alphanumeric());
                    if !dot && number {
                        Ok(Self::Int(s.parse().unwrap()))
                    } else if dot && !too_many && number {
                        Ok(Self::Float(s.parse().unwrap()))
                    } else if chars {
                        Ok(Self::Ident(base.clone()))
                    } else {
                        println!("Invalid Token: {}", s);
                        Err(format!("Invalid Token: {}", s))
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Lexer {
    s: String,
    pos: usize,
}

impl Lexer {
    pub fn new(path: PathBuf) -> Result<Self, String> {
        if let Ok(f) = File::open(path.clone()) {
            let mut s = "".to_string();
            BufReader::new(f).read_to_string(&mut s).unwrap();

            Ok(Lexer {
                s: s.to_string(),
                pos: 0,
            })
        } else {
            Err(format!("Failed to open file: {}", path.display()))
        }
    }
}

struct BlockData {
    start: char,
    end: char,
    kind: BlockKind,
}

const SINGLES: [char; 7] = ['#', '*', '$', '.', ':', '=', '@']; // '[', ']', '{', '}'];
const WHITESPACE: [char; 4] = ['#', ' ', '\n', '\t'];
const BLOCKS: [BlockData; 2] = [
    BlockData {
        start: '{',
        end: '}',
        kind: BlockKind::Curly,
    },
    BlockData {
        start: '[',
        end: ']',
        kind: BlockKind::Bracket,
    },
];

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.pos;

        while WHITESPACE
            .map(|x| Some(x))
            .iter()
            .any(|x| x == &self.s.chars().nth(self.pos))
        {
            if self.s.chars().nth(self.pos) == Some('#') {
                self.pos += 1;

                while ![Some('\n'), None].contains(&self.s.chars().nth(self.pos)) {
                    self.pos += 1;
                }
            } else {
                self.pos += 1;
            }
        }

        if let Some(blk) = BLOCKS
            .iter()
            .find(|x| Some(x.start) == self.s.chars().nth(self.pos))
        {
            // self.pos += 1;
            // let mut toks = Vec::new();

            // while self.s.chars().nth(self.pos) != Some(blk.end) {
            //     if self.s.chars().nth(self.pos) == None {
            //         return None;
            //     }

            //     if let Some(tok) = self.next() {
            //         toks.push(tok);

            //         self.pos += 1;
            //     }
            // }

            self.pos += 1;

            return Some(Token::BlockOpen(blk.kind.clone()));
        }

        if let Some(blk) = BLOCKS
            .iter()
            .find(|x| Some(x.end) == self.s.chars().nth(self.pos))
        {
            self.pos += 1;

            return Some(Token::BlockClose(blk.kind.clone()));
        }

        let mut tmp = "".to_string();

        if Some('"') == self.s.chars().nth(self.pos) {
            self.pos += 1;
            tmp.push('"');
            while Some('"') != self.s.chars().nth(self.pos) {
                tmp.push(self.s.chars().nth(self.pos).unwrap());
                self.pos += 1;
            }
            self.pos += 1;
            tmp.push('"');
        } else if SINGLES
            .map(|x| Some(x))
            .iter()
            .any(|x| x == &self.s.chars().nth(self.pos))
        {
            tmp.push(self.s.chars().nth(self.pos).unwrap());
            self.pos += 1;
        } else {
            while !(SINGLES
                .map(|x| Some(x))
                .iter()
                .chain(WHITESPACE.map(|x| Some(x)).iter())
                .chain([None].iter()))
            .any(|x| x == &self.s.chars().nth(self.pos))
            {
                tmp.push(self.s.chars().nth(self.pos).unwrap());
                self.pos += 1;
            }
        }

        let result = if tmp == "" {
            None
        } else if let Ok(result) = Token::new(&tmp) {
            Some(result)
        } else {
            None
        };
        println!("{:?}", result);

        result
    }
}
