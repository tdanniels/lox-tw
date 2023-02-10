use crate::object::Object;
use crate::token::Token;

// TODO: make this more DRY -- the expression types (Binary, Grouping, etc.)
// are repeated in the `expr_struct!`, `expr_enum!`, and `visitor_trait!` expansions.

/// Given an identifier `expr` and a list of `ident, type` pairs, make:
/// - A struct for the given expression type `expr`.
/// - An impl for `S` with `new` and `make method. The `new` method takes the
///   list of `ident: type` pairs as parameters and returns the raw struct.
///   The `make` convenience method takes the same parameters and returns
///   a Box<Expr(S)>>.
macro_rules! expr_struct {
    ($expr: ident, $($field: ident, $type: ty),*) => {
        #[derive(Debug)]
        pub struct $expr {
            $(
                pub $field: $type,
            )*
        }

        impl $expr {
            pub fn new($($field: $type,)*) -> Self {
                Self { $($field,)* }
            }

            pub fn make($($field: $type,)*) -> Box<Expr> {
                Box::new(Expr::$expr($expr::new($($field,)*)))
            }
        }
    };
}

expr_struct!(Binary, left, Box<Expr>, operator, Token, right, Box<Expr>);
expr_struct!(Grouping, expression, Box<Expr>);
expr_struct!(Literal, value, Object);
expr_struct!(Unary, operator, Token, right, Box<Expr>);

macro_rules! expr_enum {
    ($($expr: ident),*) => {
        #[derive(Debug)]
        pub enum Expr {
            $(
                $expr($expr),
            )*
        }
    };
}

expr_enum!(Binary, Grouping, Literal, Unary);
