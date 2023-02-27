use crate::expr::Expr;
use crate::token::Token;

use std::rc::Rc;

crate::ast_struct!(Stmt, Block, statements, Vec<Stmt>);
crate::ast_struct!(Stmt, Expression, expression, Expr);
crate::ast_struct!(
    Stmt,
    Function,
    name,
    Rc<Token>,
    params,
    Vec<Rc<Token>>,
    body,
    Vec<Stmt>
);
crate::ast_struct!(
    Stmt,
    If,
    condition,
    Expr,
    then_branch,
    Stmt,
    else_branch,
    Option<Stmt>
);
crate::ast_struct!(Stmt, Print, expression, Expr);
crate::ast_struct!(Stmt, Var, name, Rc<Token>, initializer, Option<Expr>);
crate::ast_struct!(Stmt, While, condition, Expr, body, Stmt);

crate::ast_enum!(Stmt, Block, Expression, Function, If, Print, Var, While);
