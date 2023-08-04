use crate::{template::{EvalError, OpKind}, Template};

use super::{InfixOp, Value};

impl InfixOp {
    pub fn eval<'a>(&self, template: &Template) -> Result<Value, EvalError> {
        match self.kind {
            kind @ OpKind::Add 
            | kind @ OpKind::Sub
            | kind @ OpKind::Div
            | kind @ OpKind::Mul
            | kind @ OpKind::Pow => {
                match (template.eval_expr(&self.lhs)?, template.eval_expr(&self.rhs)?) {
                    (Value::Integer(lhs), Value::Integer(rhs)) => {
                        Ok(Value::Integer(match kind {
                            OpKind::Add => lhs + rhs,
                            OpKind::Sub => lhs - rhs,
                            OpKind::Div => lhs / rhs,
                            OpKind::Mul => lhs * rhs,
                            OpKind::Pow => lhs * rhs,
                            _ => unreachable!(),
                        }))
                    },
                    _ => Err(EvalError::InvalidType)
                }
            },
            _ => unreachable!("That's not an infix operator"),
        }
    }
}