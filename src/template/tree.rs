use crate::{Template, NodeId, GroupHandle, NodeHandle, LeafHandle, Node, Leaf, Group, AddNodeError};



impl NodeTree for Template {
    fn get_template(&self) -> &Template {
        self
    }

    fn get_template_mut(&mut self) -> &mut Template {
        self
    }

    fn get_id(&self) -> NodeId {
        0
    }
}

impl<'a> NodeTree for GroupHandle<'a> {
    fn get_template(&self) -> &Template {
        &self.template
    }

    fn get_template_mut(&mut self) -> &mut Template {
        self.template
    }

    fn get_id(&self) -> NodeId {
        self.id
    }
}

pub trait NodeTree {
    fn get_template(&self) -> &Template;
    fn get_template_mut(&mut self) -> &mut Template;
    fn get_id(&self) -> NodeId;

    fn get_handle(&mut self, path: &str) -> Option<NodeHandle> {
        let id = self.get_id();
        let template = self.get_template_mut();
        let node = template.nodes.get(&template.get_node_from(path, id)?).map(|(node, _)| node)?;

        Some(match node {
            Node::Leaf(leaf) => NodeHandle::Leaf(LeafHandle { id: leaf.id, template: template }),
            Node::Group(group) => NodeHandle::Group(GroupHandle { id: group.id, template: template }),
        })
    }

    fn get_leaf_handle(&mut self, path: &str) -> Option<LeafHandle> {
        let id = self.get_id();
        let template = self.get_template_mut();
        let node = template.nodes.get(&template.get_node_from(path, id)?).map(|(node, _)| node)?;

        match node {
            Node::Leaf(leaf) => Some(LeafHandle { id: leaf.id, template: template }),
            Node::Group(_) => None,
        }
    }

    fn get_group_handle(&mut self, path: &str) -> Option<GroupHandle> {
        let id = self.get_id();
        let template = self.get_template_mut();
        let node = template.nodes.get(&template.get_node_from(path, id)?).map(|(node, _)| node)?;

        match node {
            Node::Group(group) => Some(GroupHandle { id: group.id, template: template }),
            Node::Leaf(_) => None,
        }
    }

    fn get_node(&self, path: &str) -> Option<&Node> {
        let id = self.get_id();
        let template = self.get_template();
        template.nodes.get(&template.get_node_from(path, id)?).map(|(node, _)| node)
    }

    fn get_leaf(&self, path: &str) -> Option<&Leaf> {
        let id = self.get_id();
        let template = self.get_template();
        let node = template.nodes.get(&template.get_node_from(path, id)?).map(|(node, _)| node)?;

        match node {
            Node::Leaf(leaf) => Some(leaf),
            Node::Group(_) => None,
        }
    }

    fn get_group(&self, path: &str) -> Option<&Group> {
        let id = self.get_id();
        let template = self.get_template();
        let node = template.nodes.get(&template.get_node_from(path, id)?).map(|(node, _)| node)?;

        match node {
            Node::Group(group) => Some(group),
            Node::Leaf(_) => None,
        }
    }
    
    fn add_group(&mut self, name: &str) -> Result<GroupHandle, AddNodeError> {
        let id = self.get_id();
        let template = self.get_template_mut();
        template.add_group_to(name, id)
    }

    fn add_leaf(&mut self, name: &str, deferred: bool) -> Result<LeafHandle, AddNodeError> {
        let id = self.get_id();
        let template = self.get_template_mut();
        template.add_leaf_to(name, id, deferred)
    }
}