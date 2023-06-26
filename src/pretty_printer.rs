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

#[allow(unused)]
pub struct AstPrinter;

impl AstPrinter {
    #[allow(unused)]
    pub fn print(expr: &Expr) -> String {
        visit(expr)
    }
}

#[allow(unused)]
fn visit(expr: &Expr) -> String {
    match expr {
        Expr::Assign(ex) => parenthesize!(&ex.name.lexeme, &ex.value),
        Expr::Binary(ex) => parenthesize!(&ex.operator.lexeme, &ex.left, &ex.right),
        Expr::Call(ex) => parenthesize!("call", &ex.callee),
        Expr::Get(ex) => parenthesize!(&("get ".to_string() + &ex.name.lexeme), &ex.object),
        Expr::Grouping(ex) => parenthesize!("group", &ex.expression),
        Expr::Literal(ex) => ex.value.to_string(),
        Expr::Logical(ex) => parenthesize!(&ex.operator.lexeme, &ex.left, &ex.right),
        Expr::Set(ex) => parenthesize!(
            &("set ".to_string() + &ex.name.lexeme),
            &ex.object,
            &ex.value
        ),
        Expr::Super(ex) => ex.keyword.lexeme.to_string(),
        Expr::This(ex) => ex.keyword.lexeme.to_string(),
        Expr::Unary(ex) => parenthesize!(&ex.operator.lexeme, &ex.right),
        Expr::Variable(ex) => ex.name.lexeme.to_string(),
    }
}

#[cfg(test)]
mod test {
    use crate::expr::{Binary, Grouping, Literal, Unary};
    use crate::{object::Object, token::Token, token_type::TokenType};

    use super::*;

    #[test]
    fn print_exprs() {
        let minus = Token::new(TokenType::Minus, "-", Object::Nil, 1).into();
        let star = Token::new(TokenType::Star, "*", Object::Nil, 1).into();
        let num123 = Object::Number(123.0).into();
        let num4567 = Object::Number(45.67).into();
        let expr = Binary::make(
            Unary::make(minus, Literal::make(num123)),
            star,
            Grouping::make(Literal::make(num4567)),
        );
        assert_eq!(
            AstPrinter::print(&expr).as_str(),
            "(* (- 123) (group 45.67))"
        );
    }
}
