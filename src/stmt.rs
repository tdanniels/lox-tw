use crate::expr::Expr;
use crate::token::Token;

crate::ast_struct!(Stmt, 'a, Block, statements, Vec<Stmt<'a>>);
crate::ast_struct!(Stmt, 'a, Expression, expression, Box<Expr<'a>>);
crate::ast_struct!(
    Stmt,
    'a,
    If,
    condition,
    Box<Expr<'a>>,
    then_branch,
    Box<Stmt<'a>>,
    else_branch,
    Option<Box<Stmt<'a>>>
);
crate::ast_struct!(Stmt, 'a, Print, expression, Box<Expr<'a>>);
crate::ast_struct!(Stmt, 'a, Var, name, &'a Token, initializer, Option<Box<Expr<'a>>>);

crate::ast_enum!(Stmt, 'a, Block, Expression, If, Print, Var);
