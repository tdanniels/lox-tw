use crate::expr::Expr;

macro_rules! parenthesize {
    ($name: expr, $($expr: expr),*) => {
        {
            let mut s = String::with_capacity(16);
            s.push('(');
            s.push_str($name);
            $(
                s.push(' ');
                s.push_str(visit($expr).as_str());
            )*
            s.push(')');
            s
        }
    };
}

pub fn visit(expr: &Expr) -> String {
    match expr {
        Expr::Binary(ex) => parenthesize!(&ex.operator.lexeme, &ex.left, &ex.right),
        Expr::Grouping(ex) => parenthesize!("group", &ex.expression),
        Expr::Literal(ex) => {
            if let Some(v) = &ex.value {
                v.to_string()
            } else {
                "nil".to_string()
            }
        }
        Expr::Unary(ex) => parenthesize!(&ex.operator.lexeme, &ex.right),
    }
}

#[cfg(test)]
mod test {
    use crate::expr::{Binary, Grouping, Literal, Unary};
    use crate::{object::Object, token::Token, token_type::TokenType};

    use super::*;

    #[test]
    fn print_exprs() {
        let expr = Binary::make(
            Unary::make(
                Token::new(TokenType::MINUS, "-", None, 1),
                Literal::make(Some(Object::Number(123.0))),
            ),
            Token::new(TokenType::STAR, "*", None, 1),
            Grouping::make(Literal::make(Some(Object::Number(45.67)))),
        );
        assert_eq!(visit(&expr).as_str(), "(* (- 123) (group 45.67))");
    }
}
