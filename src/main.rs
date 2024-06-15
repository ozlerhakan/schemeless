use clap::Parser;

use crate::schema::schema_parser;
use std::fs::File;
use std::io::BufReader;
use xml::reader::{EventReader, XmlEvent};

mod schema;
#[derive(Parser, Debug)]
#[command(name = "schemeless")]
#[command(bin_name = "schemeless")]
#[command(version = "0.0.1")]
#[command(author, version, about, long_about = None)]
#[command(next_line_help = true)]
struct SchemaArgs {
    #[clap(value_parser = clap::value_parser!(String))]
    file: String,
}

fn main() -> std::io::Result<()> {
    let args = SchemaArgs::parse();
    let file = File::open(args.file)?;
    let file = BufReader::new(file);

    let parser = EventReader::new(file);
    let mut names: Vec<String> = Vec::new();
    // let mut depth = 0;
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                // println!("{:spaces$}+{name}", "", spaces = depth * 2);
                // depth += 1;
                let local_name_option = &attributes.iter().find(|x| x.name.local_name == "name");
                if let Some(local_name) = local_name_option {
                    // let local_name = &name.local_name;
                    let name_value = &local_name.value;
                    if names.contains(&name_value) {
                        panic!(
                            "Found duplicate field names {} in the schema. ",
                            &name_value
                        )
                    }
                    names.push(name_value.to_string());
                }
                schema_parser(&name, attributes);
            }
            Ok(XmlEvent::EndElement { name }) => {
                // depth -= 1;
                // println!("{:spaces$}-{name}", "", spaces = depth * 2);
            }
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
