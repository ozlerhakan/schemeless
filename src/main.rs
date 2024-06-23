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
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                schema_parser(&mut names, &name, attributes);
            }
            Ok(XmlEvent::Characters(ref data)) => {
                println!("{:?}", data);
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
