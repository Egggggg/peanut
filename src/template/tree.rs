use super::{Template, GroupHandle, NodeHandle, LeafHandle, Node, Leaf, Group, AddNodeError, MetaHandle, Handle, NodeId};

impl NodeTree for Template {}
impl<'a> NodeTree for GroupHandle<'a> {}

pub trait NodeTree: Handle {
    fn get_handle(&mut self, path: &str) -> Option<NodeHandle> {
        let id = self.get_id();
        let template = self.get_template_mut();
        let node = template.nodes.get(&template.get_node_from(path, id)?).map(|(node, _)| node)?;

        Some(match node {
            Node::Leaf(leaf) => NodeHandle::Leaf(LeafHandle { id: leaf.id, template }),
            Node::Group(group) => NodeHandle::Group(GroupHandle { id: group.id, template }),
            Node::Meta(meta) => NodeHandle::Meta(MetaHandle { id: meta.id, template }),
        })
    }

    fn get_leaf_handle(&mut self, path: &str) -> Option<LeafHandle> {
        match self.get_handle(path)? {
            NodeHandle::Leaf(leaf) => Some(leaf),
            _ => None,
        }
    }

    fn get_group_handle(&mut self, path: &str) -> Option<GroupHandle> {
        match self.get_handle(path)? {
            NodeHandle::Group(group) => Some(group),
            _ => None,
        }
    }

    fn get_node(&self, path: &str) -> Option<&Node> {
        let id = self.get_id();
        let template = self.get_template();
        template.nodes.get(&template.get_node_from(path, id)?).map(|(node, _)| node)
    }

    fn get_leaf(&self, path: &str) -> Option<&Leaf> {
        match self.get_node(path)? {
            Node::Leaf(leaf) => Some(leaf),
            _=> None,
        }
    }

    fn get_group(&self, path: &str) -> Option<&Group> {
        match self.get_node(path)? {
            Node::Group(group) => Some(group),
            _ => None,
        }
    }

    fn add_leaf(&mut self, name: &str, deferred: bool) -> Result<LeafHandle, AddNodeError> {
        let id = self.get_id();
        let template = self.get_template_mut();
        template.add_leaf_to(name, id, deferred)
    }
    
    fn add_group(&mut self, name: &str) -> Result<GroupHandle, AddNodeError> {
        let id = self.get_id();
        let template = self.get_template_mut();
        template.add_group_to(name, id)
    }
}