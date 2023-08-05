mod tree;
mod leaf;
mod handle;
mod meta;

use std::collections::HashMap;

pub use tree::NodeTree;
pub use leaf::*;
pub use handle::Handle;

/// A Node ID, used for referencing nodes
pub type NodeId = usize;

/// The number type
pub type Integer = isize;

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
    Meta(Meta),
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
    /// Cached output
    pub cached: Option<Value>,
    /// Whether the cache is valid
    pub cache_valid: bool,
    /// A deferred leaf is not evaluated until it is used by an action
    pub deferred: bool,
    /// The direct parent of this node, if any
    pub parent: Option<NodeId>,
    /// Metadata attached to this node
    pub metadata: Vec<NodeId>,
    /// Nodes this node refers to
    pub dependencies: Vec<NodeId>,
    /// Nodes that refer to this node
    pub dependents: Vec<NodeId>,
}

/// A node that can contain other nodes
#[derive(Clone, Debug)]
pub struct Group {
    /// The ID of this node, for reference by other nodes
    pub id: NodeId,
    /// Any children contained within this node
    pub children: Vec<NodeId>,
    /// The direct parent of this node, if any
    pub parent: Option<NodeId>,
    /// Metadata attached to this node
    pub metadata: Vec<NodeId>,
    /// Common meta node, only one can be attached per group
    pub common: Option<NodeId>,
}

#[derive(Debug)]
pub enum NodeHandle<'a> {
    Leaf(LeafHandle<'a>),
    Group(GroupHandle<'a>),
    Meta(MetaHandle<'a>),
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

#[derive(Debug)]
pub struct MetaHandle<'a> {
    pub id: NodeId,
    pub template: &'a mut Template,
}

#[derive(Clone, Debug)]
pub struct Meta {
    pub id: NodeId,
    pub parent: NodeId,
    pub data: Metadata,
    pub cached: Option<Value>,
    pub cache_valid: bool,
}

/// Types of metadata to tell the template what to make without making it yourself
#[derive(Clone, Copy, Debug)]
pub enum MetadataStart {
    Common,
    Sum,
    Ident,
    Concat,
    Constraint(Constraint),
}

/// Certain metadata variants can modify other nodes
#[derive(Clone, Debug)]
pub enum Metadata {
    /// Any children of this metanode will be added to all other leaves of the direct parent
    /// 
    /// Applicable to: Groups
    Common { inner: NodeId },
    /// Contains a list of integer values, which will be added together to form the direct parent's value
    /// 
    /// Applicable to: Leaves
    Sum(Vec<Integer>),
    /// Contains the identifier belonging to its direct parent
    /// 
    /// If used in a `__common` metanode, this will contain the identifier of the node it is being placed into
    /// 
    /// Applicable to: Any
    Ident,
    /// Concatenates strings
    /// 
    /// Applicable to: Groups
    Concat(Vec<Expr>),
    /// Constrains the value of its direct parent
    /// 
    /// Applicable to: Leaves
    Constraint(Constraint),
}

#[derive(Clone, Copy, Debug)]
pub enum Constraint {
    GreaterThan(Integer),
    GreaterOrEqual(Integer),
    LessThan(Integer),
    LessOrEqual(Integer),
    Equal(Integer),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AddNodeError {
    ParentNotExists,
    ParentIsLeaf,
    InvalidParent,
    NameConflict,
    InvalidName,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditLeafError {
    NotExists,
    NotLeaf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EvalError {
    NotALeaf(NodeId),
    InfiniteRecursion(NodeId),
    MissingInfo(NodeId),
    MissingDependency(NodeId),
    MissingPathDependency(String),
    InvalidIdentRef(NodeId),
    InvalidType,
    MetaType(NodeId),
    MissingParent(NodeId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EvalMetaStatus {
    Success(Value),
    Ident,
    WrongType,
    InvalidConcatElement,
    InternalEvalError(EvalError),
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
            metadata: Vec::new(),
            common: None,
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
        let common_inner: Option<NodeId>;

        if let Some(parent) = self.nodes.get_mut(&parent) {
            match parent.0 {
                Node::Group(ref mut group) => {
                    group.children.push(id);
                    return Ok(())
                },
                Node::Leaf(_) => return Err(AddNodeError::ParentIsLeaf),
                Node::Meta(ref mut meta) => match &mut meta.data {
                    Metadata::Common { inner: group_id } => common_inner = Some(*group_id),
                    _ => return Err(AddNodeError::ParentIsLeaf),
                },
            }
        } else {
            return Err(AddNodeError::ParentNotExists)
        }

        // Do this separately to avoid having multiple mutable references
        if let Some(group_id) = common_inner {
            let group = self.get_mut_group_by_id(group_id).ok_or(AddNodeError::InvalidParent)?;
            group.metadata.push(id);
        }

        Ok(())
    }

    // TODO: Make a macro for the `add_*_to` methods
    pub fn add_leaf_to(&mut self, name: &str, parent: NodeId, deferred: bool) -> Result<LeafHandle, AddNodeError> {
        if let Some(_) = self.get_node_from(name, parent) {
            return Err(AddNodeError::NameConflict);
        }

        if !self.verify_name(name) {
            return Err(AddNodeError::InvalidName)
        }
        
        let id = self.new_id();
        let leaf = Leaf {
            id,
            value_kind: ValueKind::Undefined,
            value: None,
            cached: None,
            cache_valid: false,
            deferred,
            parent: Some(parent),
            metadata: Vec::new(),
            dependencies: Vec::new(),
            dependents: Vec::new(),
        };

        self.add_child(parent, id)?;
        self.nodes.insert(id, (Node::Leaf(leaf), name.to_owned()));

        let handle = LeafHandle {
            id,
            template: self,
        };

        Ok(handle)
    }

    pub fn add_group_to(&mut self, name: &str, parent: NodeId) -> Result<GroupHandle, AddNodeError> {
        if let Some(_) = self.get_node_from(name, parent) {
            return Err(AddNodeError::NameConflict);
        }

        if !self.verify_name(name) || name == "[COMMON INNER]" {
            return Err(AddNodeError::InvalidName)
        }

        let id = self.new_id();
        let group = Group {
            id,
            children: Vec::new(),
            parent: Some(parent),
            metadata: Vec::new(),
            common: None,
        };

        self.add_child(parent, id)?;
        self.nodes.insert(id, (Node::Group(group), name.to_owned()));

        let handle = GroupHandle {
            id,
            template: self,
        };

        Ok(handle)
    }

    pub fn add_meta_to(&mut self, name: &str, parent_id: NodeId, start: MetadataStart) -> Result<MetaHandle, AddNodeError> {
        if let Some(_) = self.get_node_from(name, parent_id) {
            return Err(AddNodeError::NameConflict);
        }

        if !self.verify_name(name) {
            return Err(AddNodeError::InvalidName)
        }

        let (data, inner_group) = match start {
            MetadataStart::Common => {
                if let Some(parent) = self.get_group_by_id(parent_id) {
                    if parent.common != None {
                        return Err(AddNodeError::InvalidParent);
                    }
                } else {
                    return Err(AddNodeError::InvalidParent);
                }

                let group = Group {
                    id: self.new_id(),
                    children: Vec::new(),
                    parent: Some(parent_id),
                    metadata: Vec::new(),
                    common: None,
                };

                // The ID for this gropu will need to be set later if it's being put into a __common meta node
                (Metadata::Common { inner: group.id }, Some(group))
            },
            MetadataStart::Sum => (Metadata::Sum(Vec::new()), None),
            MetadataStart::Ident => (Metadata::Ident, None),
            MetadataStart::Concat => (Metadata::Concat(Vec::new()), None),
            MetadataStart::Constraint(constraint) => (Metadata::Constraint(constraint), None),
        };

        let id = self.new_id();

        let mut common_inner: Option<NodeId> = None;

        let parent_id = if let Some(parent) = self.nodes.get_mut(&parent_id) {
            match parent.0 {
                Node::Group(ref mut group) => {
                    group.metadata.push(id);
                    id
                },
                Node::Leaf(ref mut leaf) => {
                    leaf.metadata.push(id);
                    id
                },
                Node::Meta(ref mut meta) => match &mut meta.data {
                    Metadata::Common { inner: group_id } => {
                        common_inner = Some(*group_id);
                        *group_id
                    },
                    _ => return Err(AddNodeError::ParentIsLeaf),
                },
            }
        } else {
            return Err(AddNodeError::ParentNotExists);
        };

        // Add the node to the group within the __common meta node
        // Do this separately to avoid having multiple mutable references
        if let Some(group_id) = common_inner {
            let parent_group = self.get_mut_group_by_id(group_id).ok_or(AddNodeError::InvalidParent)?;
            parent_group.metadata.push(id);

            for id in parent_group.children.clone() {
                // Coming back to this after i think
            }
        }

        let meta = Meta {
            id,
            parent: parent_id,
            data,
            cached: None,
            cache_valid: false,
        };
        
        self.nodes.insert(id, (Node::Meta(meta), name.to_owned()));
        if let Some(inner_group) = inner_group {
            self.nodes.insert(inner_group.id, (Node::Group(inner_group), "[COMMON INNER]".to_owned()));
        }

        let handle = MetaHandle {
            id,
            template: self,
        };

        Ok(handle)
    }

    /// Gets the ID of the node found at `path` relative to `parent`
    pub fn get_node_from(&self, path: &str, parent: NodeId) -> Option<NodeId> {
        let (name, path, last) = if let Some((name, path)) = path.split_once(".") {
            (name, path, false)
        } else {
            (path, path, true)
        };

        let finder = |child_id| {
            let child_name = &self.nodes.get(child_id)?.1;

            if child_name == name {
                Some(*child_id)
            } else {
                None
            }
        };

        let id = {
            let (parent, _) = self.nodes.get(&parent)?;
            match parent {
                Node::Group(group) => group.children.iter().chain(group.metadata.iter()).find_map(finder)?,
                Node::Leaf(leaf) => leaf.metadata.iter().find_map(finder)?,
                Node::Meta(meta) => match meta.data {
                    Metadata::Common { inner: group } => return self.get_node_from(path, group),
                    _ => return None,
                }
            }
        };
        
        if last {
            Some(id)
        } else {
            self.get_node_from(path, id)
        }
    }

    fn set_leaf_value(&mut self, id: NodeId, value: Value) -> Result<(), EditLeafError> {
        let (node, _) = self.nodes.get_mut(&id).ok_or(EditLeafError::NotExists)?;
        let node = match node {
            Node::Leaf(leaf) => Ok(leaf),
            _ => Err(EditLeafError::NotLeaf),
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
            _ => Err(EditLeafError::NotLeaf),
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
            Expr::IdentRef(_) => ValueKind::String,
            Expr::InfixOp(op) => {
                (&**op).into()
            }
        }
    }

    fn verify_name(&self, name: &str) -> bool {
        !name.contains('.')
    }

    pub fn get_leaf_by_id(&self, id: NodeId) -> Option<&Leaf> {
        match self.nodes.get(&id)?.0 {
            Node::Leaf(ref leaf) => Some(leaf),
            _ => None,
        }
    }

    pub fn get_group_by_id(&self, id: NodeId) -> Option<&Group> {
        match self.nodes.get(&id)?.0 {
            Node::Group(ref group) => Some(group),
            _ => None,
        }
    }

    pub fn get_meta_by_id(&self, id: NodeId) -> Option<&Meta> {
        match self.nodes.get(&id)?.0 {
            Node::Meta(ref meta) => Some(meta),
            _ => None,
        }
    }

    fn get_mut_leaf_by_id(&mut self, id: NodeId) -> Option<&mut Leaf> {
        match self.nodes.get_mut(&id)?.0 {
            Node::Leaf(ref mut leaf) => Some(leaf),
            _ => None,
        }
    }

    fn get_mut_group_by_id(&mut self, id: NodeId) -> Option<&mut Group> {
        match self.nodes.get_mut(&id)?.0 {
            Node::Group(ref mut group) => Some(group),
            _ => None,
        }
    }

    fn get_mut_meta_by_id(&mut self, id: NodeId) -> Option<&mut Meta> {
        match self.nodes.get_mut(&id)?.0 {
            Node::Meta(ref mut meta) => Some(meta),
            _ => None,
        }
    }

    pub fn list_nodes(&self) -> Vec<&(Node, String)> {
        self.nodes.values().collect()
    }

    pub fn eval_leaf(&mut self, id: NodeId) -> Result<Value, EvalError> {
        let mut checked = Vec::new();
        let mut updates = Vec::new();
        let out = self.eval_leaf_inner(id, &mut checked, &mut updates);


        if let Ok((out, updates)) = out.clone() {
            // Get the leaf back so we can cache the output
            if let Some(leaf) = self.get_mut_leaf_by_id(id) {
                leaf.cached = Some(out);
                leaf.cache_valid = true;
            }

            for (id, value) in updates {
                if let Some(node) = self.get_mut_leaf_by_id(*id) {
                    node.cached = Some(value.clone());
                    node.cache_valid = true;
                }
            }
        }

        out.map(|(value, _)| value)
    }

    fn eval_leaf_inner<'a>(&self, id: NodeId, checked: &mut Vec<NodeId>, updates: &'a mut Vec<(NodeId, Value)>) -> Result<(Value, &'a Vec<(NodeId, Value)>), EvalError> {
        if checked.contains(&id) {
            return Err(EvalError::InfiniteRecursion(id));
        }

        let out = match &self.nodes.get(&id).ok_or(EvalError::MissingDependency(id))?.0 {
            Node::Leaf(leaf) => {
                if leaf.cache_valid {
                    if let Some(cached) = &leaf.cached {
                        return Ok((cached.clone(), updates));
                    }
                }

                match &leaf.value {
                    Some(expr) => self.eval_expr_inner(expr, checked),
                    None => return Err(EvalError::MissingInfo(id)),
                }
            },
            Node::Group(_) => return Err(EvalError::NotALeaf(id)),
            Node::Meta(meta) => {
                if meta.cache_valid {
                    if let Some(cached) = &meta.cached {
                        return Ok((cached.clone(), updates));
                    }
                }

                match self.eval_meta_inner(&meta.data, checked) {
                    EvalMetaStatus::Success(value) => Ok(value),
                    EvalMetaStatus::Ident => {
                        let mut next = self.nodes.get(&meta.parent).ok_or(EvalError::MissingParent(meta.id))?;

                        // The `__ident` meta node returns the name of the nearest non-meta parent node
                        loop {
                            if let (Node::Meta(inner), _) = next {
                                next = self.nodes.get(&inner.parent).ok_or(EvalError::MissingParent(inner.id))?;
                                continue;
                            } else if let (Node::Group(inner), name) = next {
                                if name == "[COMMON INNER]" {
                                    next = self.nodes.get(&inner.parent.unwrap()).ok_or(EvalError::MissingParent(inner.id))?;
                                    continue;
                                }

                                break;
                            }

                            break;
                        }

                        Ok(Value::String(next.1.clone()))
                    }
                    EvalMetaStatus::WrongType => Err(EvalError::MetaType(id)),
                    EvalMetaStatus::InvalidConcatElement => Err(EvalError::InvalidType),
                    EvalMetaStatus::InternalEvalError(err) => Err(err),
                }
            },
        }?;

        updates.push((id, out.clone()));

        Ok((out, &*updates))
    }

    pub fn eval_expr(&self, expr: &Expr) -> Result<Value, EvalError> {
        let mut checked = Vec::new();
        
        self.eval_expr_inner(expr, &mut checked)
    }

    fn eval_expr_inner(&self, expr: &Expr, checked: &mut Vec<NodeId>) -> Result<Value, EvalError> {
        match expr {
            Expr::Literal(literal) => return Ok(literal.clone()),
            Expr::Reference(ref_id) => self.eval_leaf_inner(*ref_id, checked, &mut Vec::new()).map(|(value, _)| value),
            Expr::IdentRef(ref_id) => {
                let referenced_path = self.eval_leaf_inner(*ref_id, checked, &mut Vec::new()).map(|(value, _)| value)?;
                
                if let Value::String(name) = referenced_path {                                    
                    let referenced_id = self.get_node_from(&name, 0).ok_or(EvalError::MissingPathDependency(name.to_owned()))?;

                    self.eval_leaf_inner(referenced_id, checked, &mut Vec::new()).map(|(value, _)| value)
                } else {
                    Err(EvalError::InvalidIdentRef(*ref_id))
                }
            },
            Expr::InfixOp(expr) => expr.eval(self),
        }
    }

    fn eval_meta_inner(&self, meta: &Metadata, checked: &mut Vec<NodeId>) -> EvalMetaStatus {
        match meta {
            Metadata::Common { inner: _ } => EvalMetaStatus::WrongType,
            Metadata::Sum(elements) => EvalMetaStatus::Success(Value::Integer(elements.iter().sum())),
            Metadata::Ident => EvalMetaStatus::Ident,
            Metadata::Concat(elements) => self.concat_meta(elements, checked),
            Metadata::Constraint(_) => EvalMetaStatus::WrongType,
        }
    }

    fn concat_meta(&self, elements: &Vec<Expr>, checked: &mut Vec<NodeId>) -> EvalMetaStatus {
        let mut out: Vec<String> = Vec::with_capacity(elements.len());
        
        for expr in elements {
            match self.eval_expr_inner(expr, checked) {
                Ok(value) => {
                    match value {
                        Value::String(value) => out.push(value),
                        _ => return EvalMetaStatus::InvalidConcatElement,
                    }
                }
                Err(err) => return EvalMetaStatus::InternalEvalError(err),
            }
        }

        let output = out.concat();

        EvalMetaStatus::Success(Value::String(output))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LeafHandle,
        GroupHandle,
        EditLeafError,
        Expr,
        InfixOp,
        OpKind,
        Template,
        AddNodeError,
        NodeTree,
    };

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
    fn leaf_set_value() -> Result<(), EditLeafError> {
        let mut template = Template::new();

        let mut node = template.add_leaf("gup", false).unwrap();
        node.set_value(53.into())?;

        assert_eq!(node.get_value(), Some(&53.into()));

        Ok(())
    }

    #[test]
    fn leaf_set_expr() -> Result<(), EditLeafError> {
        let mut template = Template::new();
        let expr = Expr::InfixOp(Box::new(InfixOp { lhs: 53.into(), rhs: 47.into(), kind: OpKind::Add }));

        let mut node = template.add_leaf("gup", false).unwrap();
        node.set_expr(expr.clone())?;

        assert_eq!(node.get_value(), Some(&expr));

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