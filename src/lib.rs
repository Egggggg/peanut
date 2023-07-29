use std::collections::HashMap;

/// A Node ID, used internally for inter-node references
pub type NodeId = u64;

/// The number type
pub type Integer = i64;

/// The whole big guy
#[derive(Clone, Debug)]
pub struct Template {
    /// All nodes in the template, with their names
    nodes: HashMap<NodeId, (Node, String)>,
    /// The ID to use for the next identifier. This will just increment
    next_id: NodeId,
}

/// A generic node
#[derive(Clone, Debug)]
pub enum Node {
    /// A node with a single value
    Leaf(Leaf),
    /// A node that may contain subnodes
    Group(Group),
    // / A node that fulfills a special purpose
    // Meta(Meta),
}

/// The global path to a node
pub type NodePath = Vec<NodeId>;

/// A single value contained within a leaf node
#[derive(Clone, Debug)]
pub enum Value {
    /// A 64 bit signed integer
    Integer(Integer),
    /// A UTF-8 string
    String(String),
    /// A list of values, can only contain a single type of value at a time
    List(Vec<Value>),
}

/// Empty values for type resolution
#[derive(Clone, Debug)]
pub enum ValueKind {
    Undefined,
    Integer,
    String,
    IntegerList,
    StringList,
}

/// An expression to be evaluated before being referenced
#[derive(Clone, Debug)]
pub enum Expr {
    Literal(Value),
    Reference(NodePath),
}

/// A node with a single value
#[derive(Clone, Debug)]
pub struct Leaf {
    /// The ID of this node, for reference by other nodes
    pub id: NodeId,
    /// The value accessible by referring to the node
    pub value: ValueKind,
    /// A deferred leaf is not evaluated until it is used by an action
    pub deferred: bool,
    /// The direct parent of this node, if any
    pub parent: Option<NodeId>,
}

/// A handle to a group in a template
#[derive(Clone, Debug)]
pub struct Group {
    /// The ID of this node, for reference by other nodes
    pub id: NodeId,
    /// Any children contained within this node
    pub children: Vec<NodeId>,
    /// The direct parent of this node, if any
    pub parent: Option<NodeId>
}

/// Metanodes are special nodes that modify their direct parents
#[derive(Clone, Debug)]
pub enum Meta {
    /// Any children of metanode will be added to all children of the direct parent
    /// 
    /// Applicable to: Groups
    Common(Group),
    /// Contains a list of integer values, which will be added together before being referenced
    /// 
    /// Applicable to: Leaves
    Sum(NodeId, Vec<Integer>),
    /// Contains the identifier belonging to its direct parent
    /// 
    /// If used in a `__common` metanode, this will contain the identifier of the node it is being placed into
    /// 
    /// Applicable to: Any
    Ident(NodeId),
    /// Combines strings sequentially
    /// 
    /// Applicable to: Groups
    Concat(NodeId, Vec<Expr>),
    /// Constrains the value of its direct parent
    /// 
    /// Applicable to: Leaves
    Constraint(Constraint),
}

/// Empty metanodes for type resolution
#[derive(Clone, Debug)]
pub enum MetaKind {
    Common,
    Sum,
    Ident,
    Concat,
    Constraint,
}

#[derive(Clone, Debug)]
pub enum Constraint {
    GreaterThan(u64),
    GreaterOrEqual(u64),
    LessThan(u64),
    LessOrEqual(u64),
    Equal(u64),
}

#[derive(Clone, Copy, Debug)]
pub enum AddNodeError {
    ParentNotExists,
    ParentIsLeaf,
}

impl Template {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            next_id: 1,
        }
    }

    fn new_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;

        id
    }

    fn add_parent(&mut self, parent: Option<NodeId>, id: NodeId) -> Result<(), AddNodeError> {
        if let Some(parent_id) = parent {
            if let Some(parent) = self.nodes.get_mut(&parent_id) {
                match parent.0 {
                    Node::Group(ref mut group) => {
                        group.children.push(id);
                        Ok(())
                    },
                    Node::Leaf(_) => Err(AddNodeError::ParentIsLeaf),
                }
            } else {
                Err(AddNodeError::ParentNotExists)
            }
        } else {
            Ok(())
        }
    }
    
    pub fn add_group(&mut self, name: &str, parent: Option<NodeId>) -> Result<NodeId, AddNodeError> {
        let id = self.new_id();
        let group = Group {
            id,
            children: Vec::new(),
            parent,
        };

        self.add_parent(parent, id)?;
        self.nodes.insert(id, (Node::Group(group), name.to_owned()));

        Ok(id)
    }


    pub fn add_node(&mut self, name: &str, parent: Option<NodeId>, deferred: bool) -> Result<NodeId, AddNodeError> {
        let id = self.new_id();
        let leaf = Leaf {
            id,
            value: ValueKind::Undefined,
            deferred,
            parent,
        };

        self.add_parent(parent, id)?;
        self.nodes.insert(id, (Node::Leaf(leaf), name.to_owned()));

        Ok(id)
    }

    pub fn find_node(&mut self, path: &str, parent: Option<NodeId>) -> Option<NodeId> {
        let (name, path, last) = if let Some((name, path)) = path.split_once(".") {
            (name, path, false)
        } else {
            (path, path, true)
        };

        let id = match parent {
            None => {
                self.nodes.iter().find_map(|(id, (_, current))| {
                    if current == name {
                        Some(*id)
                    } else {
                        None
                    }
                })?
            },
            Some(parent_id) => {
                let (parent, _) = self.nodes.get(&parent_id)?;
                let Node::Group(parent) = parent else {
                    return None;
                };

                parent.children.iter().find_map(|child_id| {
                    if self.nodes.get(child_id)?.1 == name {
                        Some(*child_id)
                    } else {
                        None
                    }
                })?
            }
        };
            
        match &self.nodes.get(&id).unwrap().0 {
            Node::Group(group) => {
                if last {
                    Some(group.id)
                } else {
                    self.find_node(path, Some(group.id))
                }
            },
            Node::Leaf(leaf) => {
                if last {
                    Some(leaf.id)
                } else {
                    None
                }
            }
        }
        
    }
}