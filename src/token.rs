use std::string::String;

#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub enum Type {
    Oparen,
    Cparen,
    Func,
    OpenList,
    CloseList,
    Number,
    StrType,
    Space,
    Empty,
    Error
}

impl Type {
    pub fn is_matching_close(&self, matching: Type) -> bool {
        match *self {
            Type::Oparen => matching == Type::Cparen,
            Type::OpenList => matching == Type::CloseList,
            _ => false
        }
    }
}

#[derive(Copy, Clone)]
pub enum LexResult {
    Pass,
    Finish,
    FinishNew, // Use char to make new token
    FinishDelete, // Don't use char to make new token
    Continue,
    Fail
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Token {
    lexed: String,
    pub tok_type: Type,
    is_lexed: bool,
    must_append: bool
}

pub trait TokenTrait {
    fn get_lexed(&self) -> String;
    fn get_status(&self) -> bool;
    fn lex_char(&mut self, new_char: char) -> LexResult;
}

impl TokenTrait for Token {
    fn get_lexed(&self) -> String {
        self.lexed.clone()
    }

    fn get_status(&self) -> bool {
        self.is_lexed
    }

    fn lex_char(&mut self, new_char: char) -> LexResult {
        if self.is_lexed {
            LexResult::Finish
        } else if self.must_append {
            self.push_escape(new_char)
        } else {
            let res = transitions::apply_transition(self, new_char);
            self.parse_lexresult(res, new_char);
            res
        }
    }
}

impl Token {
    pub fn new(lexed: char) -> Self {
        let mut tok = Token {
            lexed: String::new(),
            tok_type: Type::Empty,
            is_lexed: false,
            must_append: false
        };
        let first = transitions::first_transition(lexed);
        tok.tok_type = first.1;
        tok.parse_lexresult(first.0, lexed);
        tok
    }

    pub fn new_preset(lexed: String, tok_type: Type) -> Self {
        Token {
            lexed: lexed,
            tok_type: tok_type,
            is_lexed: true,
            must_append: false
        }
    }

    pub fn set_lexed(&mut self, string: String) {
        self.lexed = string;
    }

    fn make_error(&mut self) {
        self.tok_type = Type::Error;
        self.status_true();
    }

    fn make_finish(&mut self, new_char: char) {
        self.lexed.push(new_char);
        self.status_true();
    }

    fn status_true(&mut self) {
        self.is_lexed = true;
    }

    fn append_true(&mut self) {
        self.must_append = true;
    }

    fn append_false(&mut self) {
        self.must_append = false;
    }

    fn parse_lexresult(&mut self, result: LexResult, lexed: char) {
        match result {
            LexResult::Pass => self.lexed.push(lexed),
            LexResult::Fail => self.make_error(),
            LexResult::Finish => self.make_finish(lexed),
            LexResult::FinishNew => self.status_true(),
            LexResult::FinishDelete => self.status_true(),
            LexResult::Continue => self.append_true()
        }
    }

    fn push_escape(&mut self, escape: char) -> LexResult {
        self.lexed.push(match escape {
            'n' => '\n',
            't' => '\t',
            'r' => '\r',
            _ => escape
        });
        self.append_false();
        LexResult::Pass
    }
}

mod transitions {
    use token::{LexResult, Token, TokenTrait, Type};

    pub fn apply_transition(tok: &Token, to_lex: char) -> LexResult {
        match tok.tok_type {
            Type::Func => func(to_lex),
            Type::Number => number(tok.get_lexed(), to_lex),
            Type::StrType => str_type(to_lex),
            _ => LexResult::Fail
        }
    }

    pub fn first_transition(to_lex: char) -> (LexResult, Type) {
        match to_lex {
            '"' => (LexResult::Continue, Type::StrType),
            '(' => (LexResult::Finish, Type::Oparen),
            ')' => (LexResult::Finish, Type::Cparen),
            ' ' => (LexResult::Finish, Type::Space),
            '0'...'9' | '.' | '-' => (LexResult::Pass, Type::Number),
            '<' => (LexResult::Finish, Type::OpenList),
            '>' => (LexResult::Finish, Type::CloseList),
            '\0' | '\n' => (LexResult::Fail, Type::Error),
            _ => (LexResult::Pass, Type::Func)
        }
    }

    fn func(to_lex: char) -> LexResult {
        if is_delimiter(to_lex) {
            LexResult::FinishNew
        } else if to_lex == '\n' || to_lex == '\r' || to_lex == '\t' {
            LexResult::Fail
        } else {
            LexResult::Pass
        }
    }

    fn number(cur_lexed: String, to_lex: char) -> LexResult {
        if to_lex == '-' {
            LexResult::Fail
        } else if to_lex == '.' {
            if check_decimal(cur_lexed) {
                LexResult::Fail
            } else {
                LexResult::Pass
            }
        } else if '0' <= to_lex && to_lex <= '9'  {
            LexResult::Pass
        } else if is_delimiter(to_lex) {
            LexResult::FinishNew
        } else {
            LexResult::Fail
        }
    }

    fn str_type(to_lex: char) -> LexResult {
        if '\\' == to_lex {
            LexResult::Continue
        } else if '"' == to_lex {
            LexResult::FinishDelete
        } else {
            LexResult::Pass
        }
    }

    fn check_decimal(num_str: String) -> bool {
        let mut has_decimal = false;
        for digit in num_str.chars() {
            if digit == '.' {
                has_decimal = true;
                break;
            }
        }
        has_decimal
    }

    fn is_delimiter(delim: char) -> bool {
        match delim {
            ' ' | ')' | '|' => true,
            _ => false
        }
    }
}
