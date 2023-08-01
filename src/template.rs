mod tree;
mod leaf;

use std::collections::HashMap;

pub use tree::NodeTree;
pub use leaf::*;

/// A Node ID, used for referencing nodes
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
#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AddNodeError {
    ParentNotExists,
    ParentIsLeaf,
    NameConflict,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditLeafError {
    NotExists,
    NotLeaf,
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
            value_kind: ValueKind::Undefined,
            value: None,
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

            parent.children.iter().find_map(|child_id| {
                let child_name = &self.nodes.get(child_id)?.1;

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

    fn set_leaf_value(&mut self, id: NodeId, value: Value) -> Result<(), EditLeafError> {
        let (node, _) = self.nodes.get_mut(&id).ok_or(EditLeafError::NotExists)?;
        let node = match node {
            Node::Leaf(leaf) => Ok(leaf),
            Node::Group(_) => Err(EditLeafError::NotLeaf),
        }?;

        let value_kind: ValueKind = (&value).into();

        node.value_kind = value_kind;
        node.value = Some(Expr::Literal(value));

        Ok(())
    }

    fn set_leaf_expr(&mut self, id: NodeId, expr: Expr) -> Result<(), EditLeafError> {
        let value_kind = self.check_expr_type(&expr);
        let (node, _) = self.nodes.get_mut(&id).ok_or(EditLeafError::NotExists)?;
        let node = match node {
            Node::Leaf(leaf) => Ok(leaf),
            Node::Group(_) => Err(EditLeafError::NotLeaf),
        }?;

        node.value_kind = value_kind;
        node.value = Some(expr);

        Ok(())
    }

    fn check_expr_type(&self, expr: &Expr) -> ValueKind {
        match expr {
            Expr::Literal(value) => value.into(),
            Expr::Reference(id) => {
                if let Some((Node::Leaf(node), _)) = self.nodes.get(id) {
                    node.value_kind
                } else {
                    ValueKind::Undefined
                }
            }
            Expr::InfixOp(op) => {
                (&**op).into()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{LeafHandle, GroupHandle};

    use super::{Template, AddNodeError, NodeTree};

    #[test]
    fn add_leaf() -> Result<(), AddNodeError> {
        let mut template = Template::new();

        template.add_leaf("gup", false)?;

        Ok(())
    }

    #[test]
    fn name_conflict() -> Result<(), AddNodeError> {
        let mut template = Template::new();

        template.add_leaf("gup", false)?;
        let node = template.add_leaf("gup", false);
        let node = node.map(|_| "gup");

        assert_eq!(node, Err(AddNodeError::NameConflict));

        Ok(())
    }

    #[test]
    fn parent_missing() {
        let mut template = Template::new();
        let node = template.add_leaf_to("gup", 50, false);
        let node = node.map(|_| "gup");

        assert_eq!(node, Err(AddNodeError::ParentNotExists));
    }

    #[test]
    fn parent_leaf() -> Result<(), AddNodeError> {
        let mut template = Template::new();
        let LeafHandle { id, template: _ } = template.add_leaf("gup", false)?;
        let child = template.add_leaf_to("gup", id, false);
        let child = child.map(|_| "gup");

        assert_eq!(child, Err(AddNodeError::ParentIsLeaf));

        Ok(())
    }

    #[test]
    fn get_leaf() -> Result<(), AddNodeError> {
        let mut template = Template::new();

        template.add_leaf("gup", false)?;
        let node = template.get_leaf("gup");
        let node = node.map(|_| "gup");
    
        assert_ne!(node, None);

        Ok(())
    }

    #[test]
    fn add_group() -> Result<(), AddNodeError> {
        let mut template = Template::new();

        template.add_group("gorp")?;

        Ok(())
    }

    #[test]
    fn get_group() -> Result<(), AddNodeError> {
        let mut template = Template::new();

        template.add_group("gorp")?;
        let node = template.get_group("gorp");
        let node = node.map(|_| "gup");

        assert_ne!(node, None);

        Ok(())
    }

    #[test]
    fn add_leaf_to_group() -> Result<(), AddNodeError> {
        let mut template = Template::new();

        let mut node = template.add_group("gorp")?;
        node.add_leaf("gup", false)?;

        Ok(())
    }

    #[test]
    fn add_group_to_group() -> Result<(), AddNodeError> {
        let mut template = Template::new();

        let mut first = template.add_group("gorp")?;
        first.add_group("gorp2")?;

        Ok(())
    }

    #[test]
    fn get_leaf_from_group() -> Result<(), AddNodeError> {
        let mut template = Template::new();

        let mut group = template.add_group("gorp")?;
        group.add_leaf("gup", false)?;

        let node = group.get_leaf("gup");
        let node = node.map(|_| "gup");

        assert_ne!(node, None);

        Ok(())
    }

    #[test]
    fn get_group_from_group() -> Result<(), AddNodeError> {
        let mut template = Template::new();

        let mut first = template.add_group("gorp")?;
        first.add_group("gorp2")?;

        let second = first.get_group("gorp2");
        let second = second.map(|_| "gorp2");

        assert_ne!(second, None);

        Ok(())
    }

    #[test]
    fn add_nested_groups() -> Result<(), AddNodeError> {
        let mut template = Template::new();

        let groups = ["nested1", "nested2", "nested3", "nested4", "nested5"];

        let GroupHandle { mut id, template: _ } = template.add_group(groups[0])?;

        for i in 1..groups.len() {
            GroupHandle { id, template: _ } = template.add_group_to(groups[i], id)?;
        }

        Ok(())
    }

    #[test]
    fn get_nested_group() -> Result<(), AddNodeError> {
        let mut template = Template::new();

        let groups = ["nested1", "nested2", "nested3", "nested4", "nested5"];

        let GroupHandle { mut id, template: _ } = template.add_group(groups[0])?;

        for i in 1..groups.len() {
            GroupHandle { id, template: _ } = template.add_group_to(groups[i], id)?;
        }

        let path = groups.join(".");
        let last = template.get_group(&path);
        let last = last.map(|_| "nested");

        assert_ne!(last, None);

        Ok(())
    }

    #[test]
    fn add_deep_groups() -> Result<(), AddNodeError> {
        let mut template = Template::new();

        let mut groups: Vec<String> = Vec::with_capacity(512);

        for i in 0..groups.capacity() {
            groups.push(format!("gorp{i}"));
        }

        let GroupHandle { mut id, template: _ } = template.add_group(&groups[0])?;

        for i in 1..groups.len() {
            GroupHandle { id, template: _ } = template.add_group_to(&format!("{}{i}", groups[i]), id)?;
        }

        Ok(())
    }

    #[test]
    fn find_deep_group() -> Result<(), AddNodeError> {
        let mut template = Template::new();

        let mut groups: Vec<String> = Vec::with_capacity(512);

        for i in 0..groups.capacity() {
            groups.push(format!("gorp{i}"));
        }

        let GroupHandle { mut id, template: _ } = template.add_group(&groups[0])?;

        for i in 1..groups.len() {
            GroupHandle { id, template: _ } = template.add_group_to(&groups[i], id)?;
        }

        let path = groups.join(".");
        let last = template.get_group(&path);
        let last = last.map(|_| "gorp");

        assert_ne!(last, None);

        Ok(())
    }
}