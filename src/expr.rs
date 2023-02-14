use crate::object::Object;
use crate::token::Token;

crate::ast_struct!(
    Expr,
    Binary,
    left,
    Box<Expr>,
    operator,
    Token,
    right,
    Box<Expr>
);
crate::ast_struct!(Expr, Grouping, expression, Box<Expr>);
crate::ast_struct!(Expr, Literal, value, Object);
crate::ast_struct!(Expr, Unary, operator, Token, right, Box<Expr>);

crate::ast_enum!(Expr, Binary, Grouping, Literal, Unary);
