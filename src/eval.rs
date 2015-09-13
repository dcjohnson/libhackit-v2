use ast::{Ast, AstTrait};
use token::{TokenTrait, Type};
use builtins;
use std::ops::IndexMut;
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
            scope = self.eval_node(scope);
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
                        let mut child = current.0.get_child(0).unwrap();
                        let node_val = child.node_val.unwrap();
                        match scope.find_func(&node_val.get_lexed()) {
                            Some(func) => {
                                if !self.inject_params(func, &mut current.0) {
                                    current.0 = func.body.clone();
                                }
                                self.stack.push(current);
                                true
                            },
                            None => {
                                child.node_val = Some(node_val);
                                current.0.insert_child(child, 0);
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

    fn inject_params(&mut self, func: &Func, current: &mut Ast) -> bool {
        let mut current_children = current.dump_children();
        let func_children = func.params.clone_children();
        if func_children.is_empty() {
            false
        } else {
            for child in func_children.into_iter() {
                current.push_child(builtins::generate_let_ast(child.node_val.unwrap(), current_children.remove(0)));
            }
            current.push_child(func.body.clone());
            true
        }
    }

    fn handle_builtin(&mut self, mut parent: (Ast, usize)) {
        let result = builtins::evaluate_builtin(parent.0);
        if result.is_push() {
            self.stack.push((result.unwrap_push(), parent.1));
        } else if result.is_error() {
            self.evaluated = true;
        } else if result.is_insert() {
            match self.stack.pop() {
                Some(mut new_parent) => {
                    parent.0 = result.unwrap_insert();
                    let new_index = parent.1 + 1;
                    new_parent.0.insert_child(parent.0, parent.1);
                    let new = (new_parent.0.get_child(new_index).unwrap(), new_index);
                    self.stack.push(new_parent);
                    self.stack.push(new);
                },
                None => self.evaluated = true
            }
        }
    }

    fn handle_function(&mut self, scope: &mut Scope) {
        if !self.expand_function(scope) {
            match self.stack.pop() {
                Some(mut parent) => {
                    let result = builtins::evaluate_set_funcs(parent.0);
                    if result.is_skip() {
                        parent.0 = result.unwrap_skip();
                        match parent.0.get_child(1) {
                            Some(new_child) => {
                                self.stack.push(parent);
                                self.stack.push((new_child, 1));
                            },
                            None => self.handle_builtin(parent)
                        }
                    } else if result.is_remove() {
                        parent.0 = result.unwrap_remove();
                        match self.stack.pop() {
                            Some(mut func_head) => {
                                let new_index = parent.1 + 1;
                                func_head.0.insert_child(parent.0, parent.1);
                                match func_head.0.get_child(new_index) {
                                    Some(new_ast) => {
                                        self.stack.push(func_head);
                                        self.stack.push((new_ast, new_index));
                                    },
                                    None => self.stack.push(func_head)
                                }
                            },
                            None => self.evaluated = true
                        }
                    } else if result.is_other() {
                        let set_result = builtins::evaluate_set(result.unwrap_other(), scope);
                        if set_result.is_insert() {
                            parent.0 = set_result.unwrap_insert();
                            match parent.0.get_child(1) {
                                Some(new_child) => {
                                    self.stack.push(parent);
                                    self.stack.push((new_child, 1));
                                },
                                None => self.handle_builtin(parent)
                            }
                        }
                    } else {
                        self.evaluated = true;
                    }
                },
                None => self.evaluated = true
            }
        }
    }

    fn handle_new_node(&mut self, mut scope: Scope, mut child: Ast) -> Scope {
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
                self.handle_function(&mut scope);
            } else if tok.tok_type == Type::Oparen {
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
        scope
    }

    fn eval_node(&mut self, mut scope: Scope) -> Scope {
        match self.stack.pop() {
            Some(mut current) => {
                match current.0.get_child(0) {
                    Some(child) => {
                        self.stack.push(current);
                        scope = self.handle_new_node(scope, child);
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
                                    self.handle_builtin(parent);
                                }
                            },
                            None => self.evaluated = true
                        }
                    }
                }
            },
            None => {
                match self.ast.get_child(0) {
                    Some(child) => scope = self.handle_new_node(scope, child),
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

pub struct Scope {
    pub parent: Option<Box<Scope>>,
    funcs: Vec<Func>,
    pub evaluate_scope: bool
}

impl Scope {
    pub fn new(parent: Scope) -> Self
    {
        Scope {
            parent: Some(Box::new(parent)),
            funcs: Vec::new(),
            evaluate_scope: false
        }
    }


    pub fn new_root() -> Self {
        Scope {
            parent: None,
            funcs: Vec::new(),
            evaluate_scope: false
        }
    }

    pub fn insert_func(&mut self, func: Func) {
        if self.find_func(func.get_name()).is_none() {
            self.insert_func_no_search(func);
        }
    }

    pub fn insert_func_no_search(&mut self, func: Func) {
        let loc = self.funcs.binary_search(&func);
        if loc.is_err() {
            self.funcs.insert(loc.unwrap_err(), func);
        }
    }

    pub fn find_func(&mut self, tok_str: &String) -> Option<&mut Func> {
        let loc = self.funcs.binary_search_by(|func| {
            tok_str.cmp(func.get_name())
        });
        match loc {
            Ok(index) => Some(self.funcs.index_mut(index)),
            _ => {
                match self.parent {
                    Some(ref mut parent) => parent.find_func(tok_str),
                    None => None
                }
            }
        }
    }
}

pub struct Func {
    name: String,
    pub params: Ast,
    pub body: Ast
}

impl Func {
    pub fn new(name: String, params: Ast, body: Ast) -> Self {
        Func {
            name: name,
            params: params,
            body: body
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn reset(&mut self, params: Ast, body: Ast) {
        self.params = params;
        self.body = body;
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
