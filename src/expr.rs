use crate::object::Object;
use crate::token::Token;

use std::rc::Rc;

crate::ast_struct!(Expr, Assign, name, Rc<Token>, value, Expr);
crate::ast_struct!(Expr, Binary, left, Expr, operator, Rc<Token>, right, Expr);
crate::ast_struct!(
    Expr,
    Call,
    callee,
    Expr,
    paren,
    Rc<Token>,
    arguments,
    Vec<Expr>
);
crate::ast_struct!(Expr, Grouping, expression, Expr);
crate::ast_struct!(Expr, Literal, value, Rc<Object>);
crate::ast_struct!(Expr, Logical, left, Expr, operator, Rc<Token>, right, Expr);
crate::ast_struct!(Expr, Unary, operator, Rc<Token>, right, Expr);
crate::ast_struct!(Expr, Variable, name, Rc<Token>);

crate::ast_enum!(Expr, Assign, Binary, Call, Grouping, Literal, Logical, Unary, Variable);
