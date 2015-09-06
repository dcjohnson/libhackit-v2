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
        print!("{}", child.node_val.unwrap().get_lexed());
    }
    EvalResult::Ignore
}

fn println(ast: &mut Ast) -> EvalResult<Ast> {
    for child in ast.dump_children().into_iter() {
        println!("{}", child.node_val.unwrap().get_lexed());
    }
    EvalResult::Ignore
}

fn add(mut ast: Ast) -> EvalResult<Ast> {
    let mut int: i64 = 0;
    let mut double: f64 = 0.0;
    for child in ast.dump_children().into_iter() {
        let child_val = child.node_val.unwrap().get_lexed();
        match child_val.parse::<i64>() {
            Ok(val) => int += val,
            _ => {
                match child_val.parse::<f64>() {
                    Ok(val) => double += val,
                    _ => return EvalResult::Error
                }
            }
        }
    }
    ast.node_val = Some(match double == 0.0 {
        true => Token::new_preset(int.to_string(), Type::Number),
        false => Token::new_preset((double + int as f64).to_string(), Type::Number)
    });
    EvalResult::Push(ast)
}

pub fn evaluate_builtin(mut ast: Ast) -> EvalResult<Ast> {
    match ast.get_child(0) {
        Some(child) => {
            match child.node_val.unwrap().get_lexed().borrow() {
                "print" => print(&mut ast),
                "println" => println(&mut ast),
                "add" => add(ast),
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
