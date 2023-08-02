use crate::{Template, AddNodeError, NodeTree};

mod template;
mod object;

pub use template::*;

fn main() -> Result<(), AddNodeError> {
    let ability_names = ["strength", "dexterity", "constitution", "intelligence", "wisdom", "charisma"];
    let mut template = Template::new();

    let mut ability_scores = template.add_group("ability_scores")?;
    for name in ability_names.iter() {
        ability_scores.add_leaf(name, false)?;
    }

    let mut abilities = template.add_group("abilities")?;
    for name in ability_names.iter() {
        abilities.add_leaf(name, false)?;
    }    

    for name in ability_names.iter() {
        set_modifier(name, &mut template).unwrap();
    }

    println!("{:?}", template);
    
    Ok(())
}

fn set_modifier(name: &str, template: &mut Template) -> Result<(), EditLeafError> {
    let source = template.get_leaf(&format!("ability_scores.{name}")).unwrap().id;

    let mut target = template.get_leaf_handle(&format!("abilities.{name}")).unwrap();
    target.set_expr(Expr::InfixOp(
        Box::new(
            InfixOp { 
                lhs: Expr::InfixOp(
                    Box::new(
                        InfixOp { 
                            lhs: Expr::Reference(source), 
                            rhs: Expr::Literal(Value::Integer(10)), 
                            kind: OpKind::Sub 
                        }
                    )
                ), 
                rhs: Expr::Literal(Value::Integer(2)),
                kind: OpKind::Div,
            }
        )
    ))?;

    Ok(())
}