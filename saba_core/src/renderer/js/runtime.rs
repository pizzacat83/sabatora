use core::ops::Add;

use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;

use crate::renderer::js::ast::{AdditiveOperator, Node};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeValue {
    /// <https://tc39.es/ecma262/multipage/ecmascript-data-types-and-values.html#sec-numeric-types>
    Number(u64),
}

pub struct JsRuntime {}

type Result<T> = core::result::Result<T, String>;

impl JsRuntime {
    fn new() -> Self {
        JsRuntime {}
    }

    // TODO: How should we model "exceptions"?
    // TODO: Why &Rc instead of Rc?
    fn eval(&mut self, node: &Rc<Node>) -> Result<RuntimeValue> {
        match node.as_ref() {
            Node::ExpressionStatement(expr) => self.eval(expr),
            Node::AdditiveExpression {
                operator,
                left,
                right,
            } => {
                let left_value = self.eval(left)?;
                let right_value = self.eval(right)?;

                match operator {
                    AdditiveOperator::Addition => {
                        if let (RuntimeValue::Number(left), RuntimeValue::Number(right)) =
                            (left_value, right_value)
                        {
                            Ok(RuntimeValue::Number(left + right))
                        } else {
                            Err(format!("Unsupported addition: ({left:?}) + ({right:?})"))
                        }
                    }
                    AdditiveOperator::Subtract => {
                        if let (RuntimeValue::Number(left), RuntimeValue::Number(right)) =
                            (left_value, right_value)
                        {
                            Ok(RuntimeValue::Number(left + right))
                        } else {
                            Err(format!("Unsupported subtraction: ({left:?}) - ({right:?})"))
                        }
                    }
                }
            }
            Node::NumericLiteral(num) => Ok(RuntimeValue::Number(*num)),
            node => Err(format!("Unsupported node: {node:?}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::renderer::js::{ast::JsParser, token::JsLexer};

    use super::*;

    #[test]
    fn test_num() {
        let input = "42;";
        let lexer = JsLexer::new(input.into());
        let mut parser = JsParser::new(lexer);
        let program = parser.parse();

        let mut runtime = JsRuntime::new();
        for node in program.body {
            let result = runtime.eval(&node);
            dbg!(&result);
        }
    }

    #[test]
    fn test_add_nums() {
        let input = "1 + 2;";
        let lexer = JsLexer::new(input.into());
        let mut parser = JsParser::new(lexer);
        let program = parser.parse();

        let mut runtime = JsRuntime::new();
        for node in program.body {
            let result = runtime.eval(&node);
            dbg!(&result);
        }
    }
}
