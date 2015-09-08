use ast::{Ast, AstTrait};
use token::{Type, Token, TokenTrait};
use eval::{Scope, Func};
use std::borrow::Borrow;

pub enum EvalResult<T: AstTrait> {
    Push(T),
    Insert(T),
    Ignore,
    Error
}

impl<T: AstTrait> EvalResult<T> {
    pub fn unwrap_push(self) -> T {
        match self {
            EvalResult::Push(res) => res,
            _ => panic!()
        }
    }

    pub fn unwrap_insert(self) -> T {
        match self {
            EvalResult::Insert(res) => res,
            _ => panic!()
        }
    }

    pub fn is_insert(&self) -> bool {
        match *self {
            EvalResult::Insert(_) => true,
            _ => false
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

    pub fn is_ignore(&self) -> bool {
        match *self {
            EvalResult::Ignore => true,
            _ => false
        }
    }
}

pub enum SetEval<T: AstTrait> {
    Skip(T),
    Remove(T),
    Other(T),
    Error
}

impl<T: AstTrait> SetEval<T> {
    pub fn is_skip(&self) -> bool {
        match *self {
            SetEval::Skip(_) => true,
            _ => false
        }
    }

    pub fn is_remove(&self) -> bool {
        match *self {
            SetEval::Remove(_) => true,
            _ => false
        }
    }

    pub fn is_other(&self) -> bool {
        match *self {
            SetEval::Other(_) => true,
            _ => false
        }
    }

    pub fn is_error(&self) -> bool {
        match *self {
            SetEval::Error => true,
            _ => false
        }
    }

    pub fn unwrap_skip(self) -> T {
        match self {
            SetEval::Skip(t) => t,
            _ => panic!()
        }
    }

    pub fn unwrap_remove(self) -> T {
        match self {
            SetEval::Remove(t) => t,
            _ => panic!()
        }
    }

    pub fn unwrap_other(self) -> T {
        match self {
            SetEval::Other(t) => t,
            _ => panic!()
        }
    }
}

fn print(mut ast: Ast) -> EvalResult<Ast> {
    for child in ast.dump_children().into_iter() {
        print!("{}", child.node_val.unwrap().get_lexed());
    }
    EvalResult::Ignore
}

fn println(mut ast: Ast) -> EvalResult<Ast> {
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

fn set(mut ast: Ast, scope: &mut Scope) -> EvalResult<Ast> {
    let mut children = ast.dump_children();
    let name = children.remove(0).get_child(0).unwrap().node_val.unwrap().get_lexed();
    if {
        match scope.find_func(&name) {
            Some(func_option) => {
                func_option.reset(children.remove(0).get_child(0).unwrap(), children.remove(0).get_child(0).unwrap());
                false
            },
            None => true
        }
    } {
        scope.insert_func_no_search(Func::new(name, children.remove(0), children.remove(0)));
    }
    EvalResult::Ignore
}

pub fn evaluate_set_funcs(mut ast: Ast) -> SetEval<Ast> {
    match ast.get_child(0) {
        Some(mut child) => {
            let mut node = child.node_val.unwrap();
            match node.get_lexed().borrow() {
                "set" => {
                    node.set_lexed("seteval".to_string());
                    child.node_val = Some(node);
                    ast.insert_child(child, 0);
                    SetEval::Skip(ast)
                },
                "name" => SetEval::Remove(ast),
                "params" => SetEval::Remove(ast),
                "body" => SetEval::Remove(ast.get_child(0).unwrap()),
                _ => {
                    child.node_val = Some(node);
                    ast.insert_child(child, 0);
                    SetEval::Other(ast)
                }
            }
        },
        None => SetEval::Error
    }
}

pub fn evaluate_set(mut ast: Ast, scope: &mut Scope) -> EvalResult<Ast> {
    match ast.get_child(0) {
        Some(mut child) => {
            let mut node = child.node_val.unwrap();
            match node.get_lexed().borrow() {
                "seteval" => set(ast, scope),
                _ => {
                    child.node_val = Some(node);
                    ast.insert_child(child, 0);
                    EvalResult::Insert(ast)
                }
            }
        },
        None => EvalResult::Error
    }
}

pub fn evaluate_builtin(mut ast: Ast, scope: &mut Scope) -> EvalResult<Ast> {
    match ast.get_child(0) {
        Some(child) => {
            match child.node_val.unwrap().get_lexed().borrow() {
                "print" => print(ast),
                "println" => println(ast),
                "add" => add(ast),
                _ => EvalResult::Ignore
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
