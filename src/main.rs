use clap::Parser;

use crate::schema::solr_parser;
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
    let mut depth = 0;
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                println!("{:spaces$}+{name}", "", spaces = depth * 2);
                depth += 1;
                solr_parser(name, attributes);
            }
            Ok(XmlEvent::EndElement { name }) => {
                depth -= 1;
                println!("{:spaces$}-{name}", "", spaces = depth * 2);
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
