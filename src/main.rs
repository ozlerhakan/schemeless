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
    let file = BufReader::new(&file);

    let parser = EventReader::new(file);
    let mut names: Vec<String> = Vec::new();
    let mut unique_key_exists = false;
    let mut id_field = String::new();
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                schema_parser(&mut names, &name, attributes);
                let local_name = name.local_name.as_str();
                if local_name == "uniqueKey" {
                    unique_key_exists = true;
                }
            }
            Ok(XmlEvent::Characters(ref data)) => {
                println!("{:?}", data);
                if unique_key_exists {
                    id_field = data.to_owned();
                    unique_key_exists = false;
                }
            }
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            }
            _ => {}
        }
    }
    if !id_field.is_empty() && !names.contains(&id_field.to_string()) {
        panic!(
            "Could not found the field '{}' among the field types.",
            id_field
        )
    }

    Ok(())
}
