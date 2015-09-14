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
    if ast.child_count() > 0 {
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

    } else {
        EvalResult::Error
    }
}

fn sub(mut ast: Ast) -> EvalResult<Ast> {
    if ast.child_count() > 0 {
        let mut int: i64 = 0;
        let mut double: f64 = 0.0;

        let mut children = ast.dump_children();
        let first = children.remove(0).node_val.unwrap().get_lexed();
        match first.parse::<i64>() {
            Ok(val) => int = val,
            _ => {
                match first.parse::<f64>() {
                    Ok(val) => double = val,
                    _ => return EvalResult::Error
                }
            }
        }

        for child in children.into_iter() {
            let child_val = child.node_val.unwrap().get_lexed();
            match child_val.parse::<i64>() {
                Ok(val) => int -= val,
                _ => {
                    match child_val.parse::<f64>() {
                        Ok(val) => double -= val,
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
    } else {
        EvalResult::Error
    }
}

fn mult(mut ast: Ast) -> EvalResult<Ast> {
    if ast.child_count() > 0 {
        let mut int: i64 = 1;
        let mut double: f64 = 1.0;

        for child in ast.dump_children().into_iter() {
            let child_val = child.node_val.unwrap().get_lexed();
            match child_val.parse::<i64>() {
                Ok(val) => int *= val,
                _ => {
                    match child_val.parse::<f64>() {
                        Ok(val) => double *= val,
                        _ => return EvalResult::Error
                    }
                }
            }
        }
        ast.node_val = Some(match double == 1.0 {
            true => Token::new_preset(int.to_string(), Type::Number),
            false => Token::new_preset((double * int as f64).to_string(), Type::Number)
        });
        EvalResult::Push(ast)
    } else {
        EvalResult::Error
    }
}

fn div(mut ast: Ast) -> EvalResult<Ast> {
    if ast.child_count() > 0 {
        let mut int: i64 = 1;
        let mut double: f64 = 1.0;
        let mut is_int = true;

        let mut children = ast.dump_children();
        let first = children.remove(0).node_val.unwrap().get_lexed();
        match first.parse::<i64>() {
            Ok(val) => int = val,
            _ => {
                match first.parse::<f64>() {
                    Ok(val) => {
                        double = val;
                        is_int = !is_int;
                    },
                    _ => return EvalResult::Error
                }
            }
        }

        for child in children.into_iter() {
            let child_val = child.node_val.unwrap().get_lexed();
            match child_val.parse::<i64>() {
                Ok(val) => {
                    if is_int {
                        int /= val;
                    } else {
                        double /= val as f64;
                    }
                },
                _ => {
                    match child_val.parse::<f64>() {
                        Ok(val) => {
                            if is_int {
                                is_int = !is_int;
                                double = int as f64;
                            }
                            double /= val;
                        },
                        _ => return EvalResult::Error
                    }
                }
            }
        }
        ast.node_val = Some(match is_int {
            true => Token::new_preset(int.to_string(), Type::Number),
            false => Token::new_preset(double.to_string(), Type::Number)
        });
        EvalResult::Push(ast)
    } else {
        EvalResult::Error
    }
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

fn let_eval(mut ast: Ast, scope: &mut Scope) -> EvalResult<Ast> {
    let mut children = ast.dump_children();
    let name = children.remove(0).get_child(0).unwrap().node_val.unwrap().get_lexed();
    if {
        match scope.find_func(&name) {
            Some(func_option) => {
                func_option.reset(Ast::new_null(), children.remove(0).get_child(0).unwrap());
                false
            },
            None => true
        }
    } {
        scope.insert_func_no_search(Func::new(name, Ast::new_null(), children.remove(0)));
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
                "let" => {
                    node.set_lexed("leteval".to_string());
                    child.node_val = Some(node);
                    ast.insert_child(child, 0);
                    SetEval::Skip(ast)
                }
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
            let node = child.node_val.unwrap();
            match node.get_lexed().borrow() {
                "seteval" => set(ast, scope),
                "leteval" => let_eval(ast, scope),
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

pub fn evaluate_builtin(mut ast: Ast) -> EvalResult<Ast> {
    match ast.get_child(0) {
        Some(mut child) => {
            let node = child.node_val.unwrap();
            child.node_val = Some(node);
            match child.node_val.unwrap().get_lexed().borrow() {
                "print" => print(ast),
                "println" => println(ast),
                "add" => add(ast),
                "sub" => sub(ast),
                "mult" => mult(ast),
                "div" => div(ast),
                _ => EvalResult::Ignore
            }
        },
        None => EvalResult::Error
    }
}

pub fn generate_let_ast(tok: Token, ast: Ast) -> Ast {
    let mut func_ast = Ast::new(Token::new_preset('('.to_string(), Type::Oparen));
    func_ast.push_child(Ast::new(Token::new_preset("let".to_string(), Type::Func)));
    func_ast.push_child({
        let mut func_name = Ast::new(Token::new_preset('('.to_string(), Type::Oparen));
        func_name.push_child(Ast::new(Token::new_preset("name".to_string(), Type::Func)));
        func_name.push_child(Ast::new(tok));
        func_name
    });
    func_ast.push_child({
        let mut func_body = Ast::new(Token::new_preset('('.to_string(), Type::Oparen));
        func_body.push_child(Ast::new(Token::new_preset("body".to_string(), Type::Func)));
        func_body.push_child(ast);
        func_body
    });
    func_ast
}
