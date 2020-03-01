mod parser;
mod iterator;

use iterator::Iterator;
use std::io::{self, BufReader, prelude::*};
use std::fs::File;
use std::char;

#[derive(Debug)]
pub enum PropertyType {
    Property,
    Key,
    Value,
    Whitespace,
    Comment,
    LineBreak,
    EscapedValue,
    Separator,
}

#[derive(Debug)]
pub struct PropertyValue {
    start: usize,
    end: usize,
    children: Option<Vec<PropertyValue>>,
    type_: PropertyType,
}

pub struct Properties;

impl Properties {
    pub fn new() -> Properties {
        Properties {}
    }

    // pub fn load(&mut self, entries: &[PropertyEntry]) {
    //     for _entry in entries {
    //         println!("Entry!");
    //     }
    // }

    pub fn parse(file: &File) -> io::Result<()> {
        let mut entries = Vec::new();
        let mut reader = BufReader::new(file);
        let mut contents = String::new();

        reader.read_to_string(&mut contents)?;

        let mut iter = Iterator::new(&contents);

        loop {
            let chr = match iter.peek() {
                Some(chr) => chr,
                None => break,
            };

            if chr.is_whitespace() {
                entries.push(parser::read_whitespace(&mut iter));
            } else if parser::is_comment_indicator(chr) {
                entries.push(parser::read_comment(&mut iter));
            } else {
                entries.push(parser::read_property(&mut iter));
            }
        }

        Self::build_property(&entries, &iter);

        Ok(())
    }

    fn build_property_component(value: &PropertyValue, iter: &Iterator) -> String {
        let mut component = String::new();
        let mut start = value.start;

        if value.children.is_some() {
            for child in value.children.as_ref().unwrap() {
                component.push_str(&iter.get_range(start, child.start));

                if let PropertyType::EscapedValue = child.type_ {
                    let chr = iter.get(child.start + 1).unwrap();

                    component.push(match chr {
                        't' => '\t',
                        'r' => '\r',
                        'n' => '\n',
                        'f' => '\x0c',
                        'u' => {
                            let num = u32::from_str_radix(&iter.get_range(
                                child.start + 2,
                                child.start + 6,
                            ), 16).unwrap_or(0);

                            char::from_u32(num).unwrap()
                        },
                        _ => chr,
                    });
                } else if let PropertyType::LineBreak = child.type_ {

                }

                start = child.end;
            }
        }

        component.push_str(&iter.get_range(start, value.end));
        component
    }

    fn build_property(values: &Vec<PropertyValue>, iter: &Iterator) {
        for value in values {
            if let PropertyType::Property = value.type_ {
                let children = value.children.as_ref().unwrap();
                let key = Self::build_property_component(&children[0], iter);
                let value = Self::build_property_component(&children[2], iter);

                println!("{}: {}", key, value);
            }
        }
    }
}

mod tests {
    #[test]
    fn basic_file() {
        use std::fs::File;
        use crate::Properties;

        let file = File::open("server.properties").unwrap();
        Properties::parse(&file).unwrap();
    }
}