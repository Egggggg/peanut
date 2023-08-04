use crate::{NodeTree, AddNodeError};

use super::{MetaHandle, Metadata, NodeHandle, Group, Node, LeafHandle, GroupHandle, Leaf, EditLeafError, Meta, NodeId};

#[derive(Clone, Copy, Debug)]
pub enum EditMetaError {
    WrongKind,
}

impl<'a> NodeTree for MetaHandle<'a> {}

impl<'a> MetaHandle<'a> {
    pub fn check_common(&self) -> Option<NodeId> {
        match self.template.get_meta_by_id(self.id) {
            Some(node) => match node.data {
                Metadata::Common { inner: group } => {
                    Some(group)
                },
                _ => None,
            },
            None => None,
        }
    }

    pub fn get_handle(&mut self, path: &str) -> Option<NodeHandle> {
        if let Some(group) = self.check_common() {
            let group_handle = GroupHandle { id: group, template: self.template };
            let node_handle = group_handle.get_node(path)?;

            // Gotta check which kind of node it is so we can return the right kind of handle
            match node_handle {
                Node::Leaf(leaf) => Some(NodeHandle::Leaf(LeafHandle { id: leaf.id, template: self.template })),
                Node::Group(group) => Some(NodeHandle::Group(GroupHandle { id: group.id, template: self.template })),
                Node::Meta(meta) => Some(NodeHandle::Meta(MetaHandle { id: meta.id, template: self.template })),
            }
        } else {
            None
        }
    }

    pub fn get_leaf_handle(&mut self, path: &str) -> Option<LeafHandle> {
        match self.get_handle(path)? {
            NodeHandle::Leaf(leaf) => Some(leaf),
            _ => None,
        }
    }

    pub fn get_group_handle(&mut self, path: &str) -> Option<GroupHandle> {
        match self.get_handle(path)? {
            NodeHandle::Group(group) => Some(group),
            _ => None,
        }
    }

    pub fn get_node(&self, path: &str) -> Option<&Node> {
        if let Some(group) = self.check_common() {
            self.template.nodes.get(&self.template.get_node_from(path, group)?).map(|(node, _)| node)
        } else {
            None
        }
    }

    pub fn get_leaf(&self, path: &str) -> Option<&Leaf> {
        match self.get_node(path)? {
            Node::Leaf(leaf) => Some(leaf),
            _ => None,
        }
    }

    pub fn get_group(&self, path: &str) -> Option<&Group> {
        match self.get_node(path)? {
            Node::Group(group) => Some(group),
            _ => None,
        }
    }

    pub fn add_leaf(&mut self, name: &str, deferred: bool) -> Result<LeafHandle, AddNodeError> {
        println!("Meta ID: {}", self.id);

        if let Some(group) = self.check_common() {
            let mut group_handle = GroupHandle { id: group, template: self.template };
            let leaf = group_handle.add_leaf(name, deferred)?;

            Ok(LeafHandle { id: leaf.id, template: self.template })
        } else {
            Err(AddNodeError::InvalidParent)
        }
    }
    
    pub fn add_group(&mut self, name: &str) -> Result<GroupHandle, AddNodeError> {
        if let Some(group) = self.check_common() {
            let mut group_handle = GroupHandle { id: group, template: self.template };
            let group = group_handle.add_group(name)?;

            Ok(GroupHandle { id: group.id, template: self.template })
        } else {
            Err(AddNodeError::InvalidParent)
        }
    }

    pub fn set_value(&mut self, value: Metadata) -> Result<(), EditMetaError> {
        match (&mut self.template.get_mut_meta_by_id(self.id).unwrap().data, value) {
            (Metadata::Sum(ref mut old), Metadata::Sum(new)) => {
                *old = new;
            },
            (Metadata::Concat(ref mut old), Metadata::Concat(new)) => {
                *old = new;
            },
            (Metadata::Constraint(ref mut old), Metadata::Constraint(new)) => {
                *old = new;
            }
            _ => return Err(EditMetaError::WrongKind),
        }

        Ok(())
    }
}