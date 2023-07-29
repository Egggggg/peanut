use peanut::{Template, AddNodeError};

fn main() -> Result<(), AddNodeError> {
    let mut template = Template::new();
    let mut abilities = template.add_group("abilities")?;

    abilities.add_node("strength", false)?;
    abilities.add_node("dexterity", false)?;
    abilities.add_node("constitution", false)?;
    abilities.add_node("intelligence", false)?;
    abilities.add_node("wisdom", false)?;
    abilities.add_node("charisma", false)?;

    // TODO: Let `template.find_node` be called
    let charisma = abilities.find_node("charisma");

    println!("{charisma:?}");

    let mut charismae = abilities.add_group("charismae")?;
    charismae.add_node("secret charisma",  false)?;

    let secret = abilities.find_node("charismae.secret charisma");

    println!("{secret:?}");

    // TODO: Make this work
    // let deep = ["this", "one", "goes", "so", "very", "deep", "oh", "wow", "this", "is", "long"];

    // let first = template.add_group(deep[0]);

    // deep.iter().fold(first, |prev, name| {
    //     prev?.add_group(name)
    // });

    // let deep_found = template.find_node("this.one.goes.so.very.deep.oh.wow.this.is.long");

    // println!("{deep_found:?}");
    println!("{template:?}");

    Ok(())
}
