use crate::expr::Expr;

crate::ast_struct!(Stmt, 'a, Expression, expression, Box<Expr<'a>>);
crate::ast_struct!(Stmt, 'a, Print, expression, Box<Expr<'a>>);

crate::ast_enum!(Stmt, 'a, Expression, Print);
