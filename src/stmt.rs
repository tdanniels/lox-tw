use crate::expr::Expr;
use crate::token::Token;
use crate::unique_id::unique_usize;

use gc::{Finalize, Gc, Trace};

crate::ast_struct!(Stmt, Block, statements, Vec<Stmt>);
crate::ast_struct!(Stmt, Class, name, Gc<Token>, methods, Vec<Gc<Function>>);
crate::ast_struct!(Stmt, Expression, expression, Expr);
crate::ast_struct!(
    Stmt,
    Function,
    name,
    Gc<Token>,
    params,
    Vec<Gc<Token>>,
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
crate::ast_struct!(Stmt, Return, keyword, Gc<Token>, value, Option<Expr>);
crate::ast_struct!(Stmt, Var, name, Gc<Token>, initializer, Option<Expr>);
crate::ast_struct!(Stmt, While, condition, Expr, body, Stmt);

crate::ast_enum!(Stmt, Block, Class, Expression, Function, If, Print, Return, Var, While);
