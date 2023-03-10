use crate::object::Object;
use crate::token::Token;
use crate::unique_id::unique_usize;

use gc::{Finalize, Gc, Trace};

crate::ast_struct!(Expr, Assign, name, Gc<Token>, value, Expr);
crate::ast_struct!(Expr, Binary, left, Expr, operator, Gc<Token>, right, Expr);
crate::ast_struct!(
    Expr,
    Call,
    callee,
    Expr,
    paren,
    Gc<Token>,
    arguments,
    Vec<Expr>
);
crate::ast_struct!(Expr, Grouping, expression, Expr);
crate::ast_struct!(Expr, Literal, value, Gc<Object>);
crate::ast_struct!(Expr, Logical, left, Expr, operator, Gc<Token>, right, Expr);
crate::ast_struct!(Expr, Unary, operator, Gc<Token>, right, Expr);
crate::ast_struct!(Expr, Variable, name, Gc<Token>);

crate::ast_enum!(Expr, Assign, Binary, Call, Grouping, Literal, Logical, Unary, Variable);
