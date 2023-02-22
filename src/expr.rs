use crate::object::Object;
use crate::token::Token;

crate::ast_struct!(Expr, 'a, Assign, name, &'a Token, value, Box<Expr<'a>>);
crate::ast_struct!(
    Expr,
    'a,
    Binary,
    left,
    Box<Expr<'a>>,
    operator,
    &'a Token,
    right,
    Box<Expr<'a>>
);
crate::ast_struct!(Expr, 'a, Grouping, expression, Box<Expr<'a>>);
crate::ast_struct!(Expr, 'a, Literal, value, &'a Object);
crate::ast_struct!(
    Expr,
    'a,
    Logical,
    left,
    Box<Expr<'a>>,
    operator,
    &'a Token,
    right,
    Box<Expr<'a>>
);
crate::ast_struct!(Expr, 'a, Unary, operator, &'a Token, right, Box<Expr<'a>>);
crate::ast_struct!(Expr, 'a, Variable, name, &'a Token);

crate::ast_enum!(Expr, 'a, Assign, Binary, Grouping, Literal, Logical, Unary, Variable);
