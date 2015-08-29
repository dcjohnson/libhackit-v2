use ast::{Ast, AstTrait};
use token::{Token, Type};

pub struct Parser {
    stack: Vec<Ast>,
}

pub trait ParserTrait {
    fn parse_token(&mut self, token: Token) -> bool;
    fn is_done(&self) -> bool;
    fn get_parsed_tree(&mut self) -> Option<Ast>;
}

impl ParserTrait for Parser {
    fn parse_token(&mut self, token: Token) -> bool {
        let result = match token.tok_type {
            Type::Oparen => self.open(token),
            Type::Cparen => self.close(token),
            Type::Func => self.parse_literal(token),
            Type::OpenList => self.open(token),
            Type::CloseList => self.close(token),
            Type::Number => self.parse_literal(token),
            Type::StrType => self.parse_literal(token),
            Type::Space => true,
            Type::Empty => false,
            Type::Error => false
        };
        if !result {
            self.stack.clear();
        }
        result
    }

    fn is_done(&self) -> bool {
        self.stack.len() == 1
    }

    fn get_parsed_tree(&mut self) -> Option<Ast> {
        match self.is_done() {
            true => self.stack.pop(),
            false => None
        }
    }
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            stack: vec![Ast::new_null()],
        }
    }

    fn open(&mut self, token: Token) -> bool {
        self.stack.push(Ast::new(token));
        true
    }

    fn close(&mut self, token: Token) -> bool {
        match self.stack.pop() {
            Some(child) => {
                match self.stack.pop() {
                    Some(mut parent) => {
                        let should_push = match child.node_val {
                            Some(ref tok) => tok.tok_type.is_matching_close(token.tok_type),
                            None => false
                        };
                        if should_push {
                            parent.add_child(child);
                            self.stack.push(parent);
                        }
                        should_push
                    },
                    None => true
                }
            },
            None => false
        }
    }

    fn parse_literal(&mut self, token: Token) -> bool {
        match self.stack.pop() {
            Some(mut node) => {
                let should_push = node.node_val.is_some();
                if should_push {
                    node.add_child(Ast::new(token));
                    self.stack.push(node);
                }
                should_push
            },
            None => false
        }
    }
}
