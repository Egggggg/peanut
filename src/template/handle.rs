use crate::{Template, AddNodeError};

use super::{NodeId, Node, MetaHandle, Meta, GroupHandle, LeafHandle, MetadataStart};

impl Handle for Template {
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

impl<'a> Handle for LeafHandle<'a> {
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

impl<'a> Handle for GroupHandle<'a> {
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

impl<'a> Handle for MetaHandle<'a> {
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

pub trait Handle {
    fn get_template(&self) -> &Template;
    fn get_template_mut(&mut self) -> &mut Template;
    fn get_id(&self) -> NodeId;

    fn get_meta_handle(&mut self, path: &str) -> Option<MetaHandle> {
        let id = self.get_id();
        let template = self.get_template_mut();
        let node = template.nodes.get(&template.get_node_from(path, id)?).map(|(node, _)| node)?;

        match node {
            Node::Meta(meta) => Some(MetaHandle { id: meta.id, template }),
            _ => None,
        }
    }

    fn get_meta(&self, path: &str) -> Option<&Meta> {
        let id = self.get_id();
        let template = self.get_template();
        let node = template.nodes.get(&template.get_node_from(path, id)?).map(|(node, _)| node)?;

        match node {
            Node::Meta(meta) => Some(meta),
            _ => None,
        }
    }

    fn add_meta(&mut self, name: &str, start: MetadataStart) -> Result<MetaHandle, AddNodeError> {
        let id = self.get_id();
        let template = self.get_template_mut();
        template.add_meta_to(name, id, start)
    }
}