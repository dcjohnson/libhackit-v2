use ast::{Ast, AstTrait};
use token::{Type, Token, TokenTrait};

pub fn generate_set_ast(tok: Token, ast: Ast) -> Ast {
    let mut func_ast = Ast::new(Token::new_preset('('.to_string(), Type::Cparen));
    func_ast.push_child(Ast::new(Token::new_preset("set".to_string(), Type::Func)));
    func_ast.push_child(Ast::new(tok));
    func_ast.push_child(ast);
    func_ast
}
