use peanut::{Template, AddNodeError};

fn main() -> Result<(), AddNodeError> {
    let mut template = Template::new();
    let abilities = template.add_group("abilities", None)?;
    template.add_node("strength", Some(abilities), false)?;
    template.add_node("dexterity", Some(abilities), false)?;
    template.add_node("constitution", Some(abilities), false)?;
    template.add_node("intelligence", Some(abilities), false)?;
    template.add_node("wisdom", Some(abilities), false)?;
    template.add_node("charisma", Some(abilities), false)?;

    let charisma = template.find_node("abilities.charisma", None);

    println!("{charisma:?}");

    let charismae = template.add_group("charismae", Some(abilities))?;
    template.add_node("secret charisma", Some(charismae), false)?;

    let secret = template.find_node("charismae.secret charisma", Some(abilities));

    println!("{secret:?}");

    let deep = ["this", "one", "goes", "so", "very", "deep", "oh", "wow", "this", "is", "deep"];

    

    println!("{template:?}");

    Ok(())
}
