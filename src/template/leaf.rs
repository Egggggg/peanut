use crate::{NodeId, Integer, LeafHandle, EditLeafError};

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

/// A node with a single value
#[derive(Clone, Debug)]
pub struct Leaf {
    /// The ID of this node, for reference by other nodes
    pub id: NodeId,
    /// The type of value accessible by referring to the node
    pub value_kind: ValueKind,
    /// `Some` if the leaf contains a static value or a dynamic expression
    /// 
    /// If this is a dynamic expression, it must evaluate to the type in `value_kind`
    pub value: Option<Expr>,
    /// A deferred leaf is not evaluated until it is used by an action
    pub deferred: bool,
    /// The direct parent of this node, if any
    pub parent: Option<NodeId>,
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

impl<'a> LeafHandle<'a> {
    pub fn set_value(&mut self, value: Value) -> Result<&mut Self, EditLeafError> {
        self.template.set_leaf_value(self.id, value)?;

        Ok(self)
    }

    pub fn set_expr(&mut self, expr: Expr) -> Result<&mut Self, EditLeafError> {
        self.template.set_leaf_expr(self.id, expr)?;

        Ok(self)
    }
}