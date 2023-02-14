use crate::expr::Expr;

crate::ast_struct!(Stmt, Expression, expression, Expr);
crate::ast_struct!(Stmt, Print, expression, Expr);

crate::ast_enum!(Stmt, Expression, Print);
