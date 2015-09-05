use token::{Token, Type};

#[derive(Clone)]
pub struct Ast {
    pub node_val: Option<Token>,
    child_nodes: Vec<Ast>
}

pub trait AstTrait {
    fn push_child(&mut self, ast: Ast);
    fn pop_child(&mut self) -> Option<Ast>;
    fn get_child(&mut self, index: usize) -> Option<Ast>;
    fn insert_child(&mut self, child: Ast, index: usize) -> bool;
    fn child_count(&self) -> usize;
    fn dump_children(&mut self) -> Vec<Ast>;
    fn clone_children(&self) -> Vec<Ast>;
}

impl AstTrait for Ast {
    fn push_child(&mut self, ast: Ast) {
        self.child_nodes.push(ast);
    }

    fn pop_child(&mut self) -> Option<Ast> {
        self.child_nodes.pop()
    }

    fn get_child(&mut self, index: usize) -> Option<Ast> {
        match self.child_nodes.len() > index {
            true => Some(self.child_nodes.remove(index)),
            false => None
        }
    }

    fn insert_child(&mut self, child: Ast, index: usize) -> bool {
        if self.child_nodes.len() >= index {
            self.child_nodes.insert(index, child);
            true
        } else {
            false
        }
    }

    fn child_count(&self) -> usize {
        self.child_nodes.len()
    }

    fn dump_children(&mut self) -> Vec<Ast> {
        let children = self.child_nodes.clone();
        self.child_nodes.clear();
        children
    }

    fn clone_children(&self) -> Vec<Ast> {
        self.child_nodes.clone()
    }
}

impl Ast {
    pub fn new(node: Token) -> Self {
        Ast {
            node_val: Some(node),
            child_nodes: Vec::new()
        }
    }

    pub fn new_null() -> Self {
        Ast {
            node_val: None,
            child_nodes: Vec::new()
        }
    }

    pub fn is_function(&self) -> bool {
        match self.node_val {
            Some(ref tok) => {
                match tok.tok_type {
                    Type::Func => true,
                    _ => false
                }
            },
            None => false
        }
    }
}
