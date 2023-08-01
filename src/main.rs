use crate::{Template, AddNodeError, NodeTree};

mod template;

pub use template::*;

fn main() -> Result<(), AddNodeError> {
    let mut template = Template::new();
    let mut abilities = template.add_group("abilities")?;

    abilities.add_node("strength", false)?;
    abilities.add_node("dexterity", false)?;
    abilities.add_node("constitution", false)?;
    abilities.add_node("intelligence", false)?;
    abilities.add_node("wisdom", false)?;
    abilities.add_node("charisma", false)?;

    let mut charismae = abilities.add_group("charismae")?;
    charismae.add_node("secret charisma",  false)?;

    let secret = abilities.get_leaf("charismae.secret charisma");

    println!("{secret:?}");

    let charisma = template.get_leaf("charisma");

    println!("{charisma:?}");

    let deep = ["this", "one", "goes", "so", "very", "deep", "oh", "wow", "this", "is", "long"];

    let GroupHandle { mut id, template: _ } = template.add_group(deep[0])?;

    println!("Original ID: {id}");

    for i in 1..deep.len() {
        println!("deep[{i}]: {}", deep[i]);
        GroupHandle { id, template: _ } = template.add_group_to(deep[i], id)?;
    }

    let deep_found = template.get_group("this.one.goes.so.very.deep.oh.wow.this.is.long");

    println!("{deep_found:?}");
    println!("{template:?}");

    Ok(())
}
