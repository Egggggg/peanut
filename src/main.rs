mod template;

use template::{EditLeafError, Expr, InfixOp, OpKind, Value};
pub use template::{Template, AddNodeError, NodeTree};

use crate::template::{Handle, MetadataStart, MetaHandle};

fn main() {
    let ability_names = ["strength", "dexterity", "constitution", "intelligence", "wisdom", "charisma"];
    let mut template = Template::new();

    let mut ability_scores = template.add_group("ability_scores").unwrap();
    for name in ability_names.iter() {
        ability_scores.add_leaf(name, false).unwrap();
    }

    let mut abilities = template.add_group("abilities").unwrap();
    for name in ability_names.iter() {
        abilities.add_leaf(name, false).unwrap();
    }    

    for name in ability_names.iter() {
        let mut node = template.get_leaf_handle(&format!("abilities.{name}")).unwrap();
        let mut meta = node.add_meta("mod", MetadataStart::Common).unwrap();
        let meta_id = meta.id;

        let name_id = meta.add_meta("name", MetadataStart::Ident).unwrap().id;
        let mut concat_node = meta.add_meta("source", MetadataStart::Concat).unwrap();
        concat_node.set_value(template::Metadata::Concat(vec![Expr::Literal(Value::String("ability_scores.".to_owned())), Expr::Reference(name_id)])).unwrap();
        let concat_id = concat_node.id;

        println!("Meta ID: {}", meta_id);
        println!("{:#?}", template);
        let mut meta = MetaHandle { id: meta_id, template: &mut template };
        let mut modifier = meta.add_leaf("mod", false).unwrap();
        modifier.set_expr(Expr::InfixOp(
            Box::new(
                InfixOp { 
                    lhs: Expr::InfixOp(
                        Box::new(
                            InfixOp { 
                                lhs: Expr::IdentRef(concat_id), 
                                rhs: Expr::Literal(Value::Integer(10)), 
                                kind: OpKind::Sub 
                            }
                        )
                    ), 
                    rhs: Expr::Literal(Value::Integer(2)),
                    kind: OpKind::Div,
                }
            ))
        ).unwrap();
    }

    println!("{:#?}", template);
}