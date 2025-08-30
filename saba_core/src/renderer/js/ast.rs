use core::iter::Peekable;
use core::panic;

use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;

use crate::renderer::js::token::{JsLexer, Token};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    // TODO: why Rc?
    ExpressionStatement(Rc<Node>),

    /// <https://262.ecma-international.org/#sec-additive-operators>
    AdditiveExpression {
        operator: AdditiveOperator,
        left: Rc<Node>,
        right: Rc<Node>,
    },

    NumericLiteral(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdditiveOperator {
    Addition,
    Subtract,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub body: Vec<Rc<Node>>,
}

pub struct JsParser {
    t: Peekable<JsLexer>,
}

type Result<T> = core::result::Result<T, String>;

impl JsParser {
    pub fn new(lexer: JsLexer) -> JsParser {
        JsParser {
            t: lexer.peekable(),
        }
    }

    pub fn parse(&mut self) -> Program {
        let mut body = Vec::new();

        loop {
            let node = self.source_element();

            match node {
                Ok(n) => body.push(n),
                Err(error) => {
                    #[cfg(test)]
                    println!("error: {error:?}");

                    // TODO: need to distinguish "End of program" and "invalid syntax"
                    return Program { body };
                }
            }
        }
    }

    fn source_element(&mut self) -> Result<Rc<Node>> {
        self.statement()
    }

    fn statement(&mut self) -> Result<Rc<Node>> {
        self.expression_statement()
    }

    fn expression_statement(&mut self) -> Result<Rc<Node>> {
        if let Ok(expr) = self.assignment_expression() {
            match self.t.next() {
                Some(Token::Punctuator(';')) => Ok(Rc::new(Node::ExpressionStatement(expr))),
                token => Err(format!(
                    "expression_statement: ';' needed but {token:?} found instead"
                )),
            }
        } else {
            Err("expression_statement: cannot parse".into())
        }
    }

    fn assignment_expression(&mut self) -> Result<Rc<Node>> {
        self.additive_expression()
    }

    fn additive_expression(&mut self) -> Result<Rc<Node>> {
        let left = self.left_hand_side_expression()?;

        // Peek and check if AdditiveOperator comes next.
        // If it is, consume it.
        // If not, don't consume, and just return.
        let operator = match self.t.peek() {
            Some(Token::Punctuator('+')) => {
                self.t.next();
                AdditiveOperator::Addition
            }
            Some(Token::Punctuator('-')) => {
                self.t.next();
                AdditiveOperator::Subtract
            }
            _ => {
                return Ok(left);
            }
        };

        let right = self.additive_expression()?;
        Ok(Rc::new(Node::AdditiveExpression {
            operator,
            left,
            right,
        }))
    }

    fn left_hand_side_expression(&mut self) -> Result<Rc<Node>> {
        self.member_expression()
    }

    fn member_expression(&mut self) -> Result<Rc<Node>> {
        self.primary_expression()
    }

    fn primary_expression(&mut self) -> Result<Rc<Node>> {
        self.literal()
    }

    fn literal(&mut self) -> Result<Rc<Node>> {
        match self.t.next() {
            Some(Token::Number(num)) => Ok(Rc::new(Node::NumericLiteral(num))),
            token => Err(format!("literal: Invalid token {token:?}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_nums() {
        let input = "1 + 2;".into();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser {
            t: lexer.peekable(),
        };
        let program = parser.parse();
        dbg!(&program);
    }
}
