use ast::{Ast, AstTrait};
use token::{Type, Token, TokenTrait};
use std::borrow::Borrow;

pub enum EvalResult<T: AstTrait> {
    Push(T),
    Ignore,
    Error
}

impl<T: AstTrait> EvalResult<T> {
    pub fn unwrap(self) -> T {
        match self {
            EvalResult::Push(res) => res,
            _ => panic!()
        }
    }

    pub fn is_push(&self) -> bool {
        match *self {
            EvalResult::Push(_) => true,
            _ => false
        }
    }

    pub fn is_error(&self) -> bool {
        match *self {
            EvalResult::Error => true,
            _ => false
        }
    }
}

fn print(ast: &mut Ast) -> EvalResult<Ast> {
    for child in ast.dump_children().into_iter() {
        println!("{}", child.node_val.unwrap().get_lexed());
    }
    EvalResult::Ignore
}

pub fn evaluate_builtin(mut ast: Ast) -> EvalResult<Ast> {
    match ast.get_child(0) {
        Some(child) => {
            match child.node_val.unwrap().get_lexed().borrow() {
                "print" => print(&mut ast),
                _ => EvalResult::Error
            }
        },
        None => EvalResult::Error
    }
}

pub fn generate_set_ast(tok: Token, ast: Ast) -> Ast {
    let mut func_ast = Ast::new(Token::new_preset('('.to_string(), Type::Cparen));
    func_ast.push_child(Ast::new(Token::new_preset("set".to_string(), Type::Func)));
    func_ast.push_child(Ast::new(tok));
    func_ast.push_child(ast);
    func_ast
}
