mod ops;

use super::{NodeId, Integer, LeafHandle, EditLeafError, Node, EvalError};

/// A single value contained within a leaf node
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    /// A 64 bit signed integer
    Integer(Integer),
    /// A UTF-8 string
    String(String),
    /// A list of values
    List(Vec<Expr>),
}

/// Empty values for type resolution
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueKind {
    Undefined,
    Integer,
    String,
    List,
}

/// An expression to be evaluated before being referenced
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Literal(Value),
    Reference(NodeId),
    /// This one can be used to reference whatever has the name contained in the referenced node
    IdentRef(NodeId),
    InfixOp(Box<InfixOp>),
}

/// An operation with a left hand side (lhs) and a right hand side (rhs)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InfixOp {
    pub lhs: Expr,
    pub rhs: Expr,
    pub kind: OpKind,
}

/// Types of operations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpKind {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Neg,
}

impl From<&Value> for ValueKind {
    fn from(value: &Value) -> Self {
        match value {
            Value::Integer(_) => ValueKind::Integer,
            Value::String(_) => ValueKind::String,
            Value::List(_) => ValueKind::List,
        }
    }
}

impl From<&InfixOp> for ValueKind {
    fn from(value: &InfixOp) -> Self {
        // For now infix ops can only be used on integers
        ValueKind::Integer
    }
}

impl From<Integer> for Value {
    fn from(value: Integer) -> Self {
        Value::Integer(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<Value> for Expr {
    fn from(value: Value) -> Self {
        Expr::Literal(value)
    }
}

impl From<Integer> for Expr {
    fn from(value: Integer) -> Self {
        let i: Value = value.into();
        i.into()
    }
}

impl From<String> for Expr {
    fn from(value: String) -> Self {
        let i: Value = value.into();
        i.into()
    }
}

impl<'a> LeafHandle<'a> {
    pub fn set_value(&mut self, value: Value) -> Result<&mut Self, EditLeafError> {
        self.template.set_leaf_value(self.id, value)?;

        Ok(self)
    }

    pub fn set_expr(&mut self, expr: Expr) -> Result<&mut Self, EditLeafError> {
        self.template.set_leaf_expr(self.id, expr)?;

        Ok(self)
    }

    pub fn get_value(&self) -> Option<&Expr> {
        match &self.template.nodes.get(&self.id)?.0 {
            Node::Leaf(leaf) => {
                (&leaf).value.as_ref()
            },
            _ => None
        }
    }

    pub fn eval(&mut self) -> Result<Value, EvalError>{
        self.template.eval_leaf(self.id)
    }
}