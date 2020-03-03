mod parser;
mod iterator;

use iterator::Iterator;
use std::io::{self, BufReader, prelude::*};
use std::collections::HashMap;
use std::fs::File;
use std::char;

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

pub struct PropertyValue {
    start: usize,
    end: usize,
    children: Option<Vec<PropertyValue>>,
    type_: PropertyType,
}

pub struct Properties {
    contents: String,
    values: Vec<PropertyValue>,
    data: HashMap<String, String>,
}

impl Properties {
    pub fn new(contents: &String) -> Properties {
        let contents = contents.clone();
        let mut iter = Iterator::new(&contents);
        let values = Self::build_property_values(&mut iter);
        let data = Self::build_properties(&values, &iter);

        Properties {
            contents,
            values,
            data,
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    pub fn set(&mut self, key: &str, value: String) {
        self.data.insert(key.to_string(), value);
    }

    pub fn new_file(file: &File) -> io::Result<Properties> {
        let mut reader = BufReader::new(file);
        let mut contents = String::new();

        reader.read_to_string(&mut contents)?;

        Ok(Self::new(&contents))
    }

    pub fn parse_file(file: &File) -> io::Result<HashMap<String, String>> {
        let mut reader = BufReader::new(file);
        let mut contents = String::new();

        reader.read_to_string(&mut contents)?;

        Ok(Self::parse(&contents))
    }

    pub fn parse(contents: &String) -> HashMap<String, String>  {
        let mut iter = Iterator::new(contents);
        let entries = Self::build_property_values(&mut iter);

        Self::build_properties(&entries, &iter)
    }

    fn build_property_values(iter: &mut Iterator) -> Vec<PropertyValue> {
        let mut values = Vec::new();

        loop {
            let chr = match iter.peek() {
                Some(chr) => chr,
                None => break,
            };

            if chr.is_whitespace() {
                values.push(parser::read_whitespace(iter));
            } else if parser::is_comment_indicator(chr) {
                values.push(parser::read_comment(iter));
            } else {
                values.push(parser::read_property(iter));
            }
        }

        values
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

    fn build_properties(values: &Vec<PropertyValue>, iter: &Iterator) -> HashMap<String, String> {
        let mut data = HashMap::new();

        for value in values {
            if let PropertyType::Property = value.type_ {
                let children = value.children.as_ref().unwrap();
                let key = Self::build_property_component(&children[0], iter);
                let value = Self::build_property_component(&children[2], iter);

                data.insert(key, value);
            }
        }

        data
    }
}