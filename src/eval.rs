use ast::{Ast, AstTrait};
use token::{Token, TokenTrait, Type};
use builtins;
use std::cmp::Ordering;

pub struct Eval {
    ast: Ast,
    stack: Vec<(Ast, usize)>,
    evaluated: bool
}

pub trait EvalTrait {
    fn eval(&mut self);
    fn is_evaluated(&self) -> bool;
}

pub trait PrettyPrint {
    fn pretty_print(&mut self) -> String;
    fn push_whitespace(&self, pretty: &mut String);
}

impl PrettyPrint for Eval {
    fn push_whitespace(&self, pretty: &mut String) {
        for _ in 0..self.stack.len() {
            pretty.push('\t');
        }
    }

    fn pretty_print(&mut self) -> String {
        let mut pretty = String::new();
        loop {
            match self.stack.pop() {
                Some(mut current) => {
                    match current.0.get_child(0) {
                        Some(mut child) => {
                            self.stack.push(current);
                            if child.node_val.is_some() {
                                let tok = child.node_val.unwrap();
                                if tok.tok_type == Type::Oparen {
                                    self.push_whitespace(&mut pretty);
                                    pretty.push('(');
                                    pretty.push('\n');
                                } else if tok.tok_type == Type::OpenList {
                                    self.push_whitespace(&mut pretty);
                                    pretty.push('<');
                                    pretty.push('\n');
                                }
                                child.node_val = Some(tok);
                            }
                            self.stack.push((child, 0));
                        },
                        None => {
                            if current.0.node_val.is_some() {
                                let tok = current.0.node_val.unwrap();
                                self.push_whitespace(&mut pretty);
                                match tok.tok_type {
                                    Type::Oparen => pretty.push(')'),
                                    Type::OpenList => pretty.push('>'),
                                    _ => pretty.push_str(&tok.get_lexed())
                                }
                            }
                            pretty.push('\n');
                        }
                    }
                },
                None => {
                    match self.ast.get_child(0) {
                        Some(mut child) => {
                            if child.node_val.is_some() {
                                let tok = child.node_val.unwrap();
                                if tok.tok_type == Type::Oparen {
                                    self.push_whitespace(&mut pretty);
                                    pretty.push('(');
                                    pretty.push('\n');
                                } else if tok.tok_type == Type::OpenList {
                                    self.push_whitespace(&mut pretty);
                                    pretty.push('<');
                                    pretty.push('\n');
                                }
                                child.node_val = Some(tok);
                            }
                            self.stack.push((child, 0));
                        },
                        None => break
                    }
                }
            }
            if self.stack.is_empty() && self.ast.child_count() == 0 {
                break;
            }
        }
        pretty
    }
}

impl EvalTrait for Eval {
    fn eval(&mut self) {
        let mut scope = Scope::new_root();
        while !self.evaluated {
            scope = self.eval_node(scope); // If scope says evaluate, evaluate.
        }
    }

    fn is_evaluated(&self) -> bool {
        self.evaluated
    }
}

impl Eval {
    pub fn new(ast: Ast) -> Self {
        Eval {
            ast: ast,
            stack: Vec::new(),
            evaluated: false
        }
    }

    pub fn new_option(ast: Option<Ast>) -> Self {
        match ast {
            Some(tree) => Eval {
                ast: tree,
                stack: Vec::new(),
                evaluated: false
            },
            None => panic!()
        }
    }

    fn expand_function(&mut self, scope: &mut Scope) -> bool {
        match self.stack.pop() {
            Some(mut current) => {
                match current.0.is_function() {
                    true => {
                        let unwraped_current = current.0.node_val.unwrap();
                        match scope.find_func(&unwraped_current.get_lexed()) {
                            Some(ref func) => self.inject_params(func),
                            None => {
                                current.0.node_val = Some(unwraped_current);
                                self.stack.push(current);
                                false
                            }
                        }
                    },
                    false => {
                        self.stack.push(current);
                        false
                    }
                }
            },
            None => false
        }
    }

    fn inject_params(&mut self, func: &Func) -> bool {
        match self.stack.pop() {
            Some(mut current) => {
                let mut current_children = current.0.dump_children();
                let func_children = func.params.clone_children();
                for child in func_children.into_iter() {
                    current.0.push_child(builtins::generate_set_ast(child.node_val.unwrap(), current_children.remove(0)));
                }
                current.0.push_child(func.body.clone());
                self.stack.push(current);
                true
            },
            None => false
        }
    }

    fn eval_node(&mut self, mut scope: Scope) -> Scope {
        match self.stack.pop() {
            Some(mut current) => {
                match current.0.get_child(0) {
                    Some(mut child) => {
                        self.stack.push(current);
                        if child.node_val.is_some() {
                            let tok = child.node_val.unwrap();
                            if tok.tok_type == Type::Func {
                                child.node_val = Some(tok);
                                match self.stack.pop() {
                                    Some(mut parent) => {
                                        parent.0.insert_child(child, 0);
                                        self.stack.push(parent);
                                    },
                                    None => panic!()
                                }
                                if !self.expand_function(&mut scope) {
                                    let mut func = self.stack.pop().unwrap();
                                    let result = builtins::evaluate_builtin(func.0);
                                    if result.is_push() {
                                        self.stack.push((result.unwrap(), func.1));
                                    } else if result.is_error() {
                                        self.evaluated = true;
                                    }
                                }
                            } else if tok.tok_type == Type::Cparen {
                                child.node_val = Some(tok);
                                self.stack.push((child, 0));
                                scope = Scope::new(scope);
                            } else {
                                child.node_val = Some(tok);
                                self.stack.push((child, 0));
                            }
                        } else {
                            self.stack.push((child, 0));
                        }
                    },
                    None => {
                        match self.stack.pop() {
                            Some(mut parent) => {
                                let new_index = current.1 + 1;
                                parent.0.insert_child(current.0, current.1);
                                if parent.0.child_count() > new_index {
                                    let new_child = parent.0.get_child(new_index);
                                    self.stack.push(parent);
                                    self.stack.push((new_child.unwrap(), new_index));
                                } else {
                                    self.stack.push(parent);
                                    scope.evaluate_scope = true;
                                }
                            },
                            None => self.evaluated = true
                        }
                    }
                }
            },
            None => {
                match self.ast.get_child(0) {
                    Some(mut child) => {
                        if child.node_val.is_some() {
                            let tok = child.node_val.unwrap();
                            if tok.tok_type == Type::Func {
                                child.node_val = Some(tok);
                                match self.stack.pop() {
                                    Some(mut parent) => {
                                        parent.0.insert_child(child, 0);
                                        self.stack.push(parent);
                                    },
                                    None => panic!()
                                }
                                if !self.expand_function(&mut scope) {
                                    let mut func = self.stack.pop().unwrap();
                                    let result = builtins::evaluate_builtin(func.0);
                                    if result.is_push() {
                                        self.stack.push((result.unwrap(), func.1));
                                    } else if result.is_error() {
                                        self.evaluated = true;
                                    }
                                }
                            } else if tok.tok_type == Type::Cparen {
                                child.node_val = Some(tok);
                                self.stack.push((child, 0));
                                scope = Scope::new(scope);
                            } else {
                                child.node_val = Some(tok);
                                self.stack.push((child, 0));
                            }
                        } else {
                            self.stack.push((child, 0));
                        }
                    },
                    None => self.evaluated = true
                }
            }
        }
        if self.stack.is_empty() && self.ast.child_count() == 0 {
            self.evaluated = true;
        }
        scope
    }
}

struct Scope {
    pub parent: Option<Box<Scope>>,
    funcs: Vec<Func>,
    in_builtin: bool,
    pub evaluate_scope: bool
}

impl Scope {
    pub fn new(parent: Scope) -> Self
    {
        Scope {
            parent: Some(Box::new(parent)),
            funcs: Vec::new(),
            in_builtin: false,
            evaluate_scope: false
        }
    }


    pub fn new_root() -> Self {
        Scope {
            parent: None,
            funcs: Vec::new(),
            in_builtin: false,
            evaluate_scope: false
        }
    }

    pub fn insert_func(&mut self, func: Func) {
        if self.find_func(func.get_name()).is_none() {
            let loc = self.funcs.binary_search(&func);
            if loc.is_err() {
                self.funcs.insert(loc.unwrap(), func);
            }
        }
    }

    pub fn find_func(&mut self, tok_str: &String) -> Option<Func> {
        let loc = self.funcs.binary_search_by(|func| {
            tok_str.cmp(func.get_name())
        });
        match loc {
            Ok(index) => Some(self.funcs.remove(index)),
            _ => {
                match self.parent {
                    Some(ref mut parent) => parent.find_func(tok_str),
                    None => None
                }
            }
        }
    }
}

struct Func {
    name: String,
    pub body: Ast,
    pub params: Ast
}

impl Func {
    pub fn new(name: String, body: Ast, params: Ast) -> Self {
        Func {
            name: name,
            body: body,
            params: params
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }
}

impl Ord for Func {
    fn cmp(&self, other: &Func) -> Ordering {
        self.name.cmp(other.get_name())
    }
}

impl PartialOrd for Func {
    fn partial_cmp(&self, other: &Func) -> Option<Ordering> {
        self.name.partial_cmp(other.get_name())
    }
}

impl Eq for Func { }

impl PartialEq for Func {
    fn eq(&self, other: &Func) -> bool {
        self.name.eq(other.get_name())
    }

    fn ne(&self, other: &Func) -> bool {
        self.name.ne(other.get_name())
    }
}
