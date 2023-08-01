mod tree;

use std::{collections::HashMap, rc::Rc, cell::RefCell};

pub use tree::NodeTree;

/// A Node ID, used internally for inter-node references
pub type NodeId = u64;

/// The number type
pub type Integer = i64;

/// The whole big guy
#[derive(Clone, Debug)]
pub struct Template {
    /// All nodes in the template by ID
    nodes: HashMap<NodeId, (Node, String)>,
    /// The ID to use for the next ID. This will just increment
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

/// A node that can contain other nodes
#[derive(Clone, Debug)]
pub struct Group {
    /// The ID of this node, for reference by other nodes
    pub id: NodeId,
    /// Any children contained within this node
    pub children: Vec<NodeId>,
    /// The direct parent of this node, if any
    pub parent: Option<NodeId>
}

#[derive(Debug)]
pub enum NodeHandle<'a> {
    Leaf(LeafHandle<'a>),
    Group(GroupHandle<'a>),
}

/// A handle to a leaf node
#[derive(Debug)]
pub struct LeafHandle<'a> {
    pub id: NodeId,
    pub template: &'a mut Template,
}

/// A handle to a group node
#[derive(Debug)]
pub struct GroupHandle<'a> {
    pub id: NodeId,
    pub template: &'a mut Template,
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
    NameConflict,
}

impl Template {
    pub fn new() -> Self {
        let mut template = Self {
            nodes: HashMap::new(),
            next_id: 1,
        };

        let mother_group = Group {
            id: 0,
            children: Vec::new(),
            parent: None,
        };

        template.nodes.insert(0, (Node::Group(mother_group), "[THE MOTHER]".to_owned()));

        template
    }

    fn new_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;

        id
    }

    fn add_child(&mut self, parent: NodeId, id: NodeId) -> Result<(), AddNodeError> {
        if let Some(parent) = self.nodes.get_mut(&parent) {
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
    }

    pub fn add_group_to(&mut self, name: &str, parent: NodeId) -> Result<GroupHandle, AddNodeError> {
        if let Some(_) = self.get_node_from(name, parent) {
            return Err(AddNodeError::NameConflict);
        }

        let id = self.new_id();
        let group = Group {
            id,
            children: Vec::new(),
            parent: Some(parent),
        };

        self.add_child(parent, id)?;
        self.nodes.insert(id, (Node::Group(group), name.to_owned()));

        let handle = GroupHandle {
            id,
            template: self,
        };

        Ok(handle)
    }

    pub fn add_leaf_to(&mut self, name: &str, parent: NodeId, deferred: bool) -> Result<LeafHandle, AddNodeError> {
        if let Some(_) = self.get_node_from(name, parent) {
            return Err(AddNodeError::NameConflict);
        }
        
        let id = self.new_id();
        let leaf = Leaf {
            id,
            value: ValueKind::Undefined,
            deferred,
            parent: Some(parent),
        };

        self.add_child(parent, id)?;
        self.nodes.insert(id, (Node::Leaf(leaf), name.to_owned()));

        let handle = LeafHandle {
            id,
            template: self,
        };

        Ok(handle)
    }

    pub fn get_node_from(&self, path: &str, parent: NodeId) -> Option<NodeId> {
        let (name, path, last) = if let Some((name, path)) = path.split_once(".") {
            (name, path, false)
        } else {
            (path, path, true)
        };

        let id = {
            let (parent, _) = self.nodes.get(&parent)?;
            let Node::Group(ref parent) = *parent else {
                return None;
            };

            println!("{}'s children: {:?}", parent.id, parent.children);

            parent.children.iter().find_map(|child_id| {
                let child_name = &self.nodes.get(child_id)?.1;
                println!("{child_name} vs {name}");
                if child_name == name {
                    Some(*child_id)
                } else {
                    None
                }
            })?
        };

        let node = &self.nodes.get(&id).unwrap().0;
            
        match node {
            Node::Group(group) => {
                if last {
                    Some(group.id)
                } else {
                    self.get_node_from(path, group.id)
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