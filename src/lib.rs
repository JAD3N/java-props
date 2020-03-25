#[macro_use]
extern crate lazy_static;

mod parser;
mod iterator;
mod utils;

use iterator::Iterator;
use std::io::{self, BufReader, prelude::*};
use std::collections::HashMap;
use std::fs::File;
use std::char;

#[derive(PartialEq, Debug)]
pub enum PropertyType {
    Property,
    Key,
    Value,
    Whitespace,
    Comment,
    LineBreak,
    EscapedValue,
    Separator,
    Raw,
}

#[derive(Debug)]
pub struct PropertyRange {
    start: usize,
    end: usize,
}

#[derive(Debug)]
pub enum PropertyData {
    Range(PropertyRange),
    Text(String),
}

#[derive(Debug)]
pub struct PropertyValue {
    data: PropertyData,
    children: Option<Vec<PropertyValue>>,
    type_: PropertyType,
}

#[derive(Debug)]
pub struct Properties {
    contents: String,
    values: Vec<PropertyValue>,
    value_map: HashMap<String, usize>,
    data: HashMap<String, String>,
}

impl Properties {
    pub fn new() -> Properties {
        Properties {
            contents: String::new(),
            values: vec![],
            value_map: HashMap::new(),
            data: HashMap::new(),
        }
    }

    pub fn new_str(contents: &String) -> Properties {
        let contents = contents.clone();

        let mut iter = Iterator::new(&contents);
        let mut value_map = HashMap::new();

        let values = Self::build_property_values(&mut iter);
        let data = Self::build_properties(&values, &iter);

        for (i, value) in values.iter().enumerate() {
            if value.type_ == PropertyType::Property {
                let key = Self::build_property_component(match &value.children {
                    Some(children) => &children[0],
                    None => continue,
                }, &iter);

                value_map.insert(key, i);
            }
        }

        Properties {
            contents,
            values,
            value_map,
            data,
        }
    }

    pub fn new_file(file: &File) -> io::Result<Properties> {
        let mut reader = BufReader::new(file);
        let mut contents = String::new();

        reader.read_to_string(&mut contents)?;

        Ok(Self::new_str(&contents))
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        let escaped_key = utils::escape_key(key);
        let escaped_value = utils::escape_value(&value);

        let seperator = "=";

        self.data.insert(String::from(key), String::from(value));

        let property_value = if self.value_map.contains_key(key) {
            let index = *self.value_map.get(key).unwrap();
            &mut self.values[index]
        } else {
            let last_value = self.values.last();
            if last_value.is_some() && Self::is_newline_value(last_value) {
                self.values.push(PropertyValue {
                    data: PropertyData::Text(String::from("\n")),
                    children: None,
                    type_: PropertyType::Raw,
                });
            }

            self.values.push(PropertyValue {
                data: PropertyData::Text(String::new()),
                children: None,
                type_: PropertyType::Raw,
            });

            self.value_map.insert(String::from(key), self.values.len() - 1);
            self.values.last_mut().unwrap()
        };

        if property_value.type_ == PropertyType::Raw {
            if let PropertyData::Text(text) = &mut property_value.data {
                // empty existing text
                text.clear();

                // set new value
                text.push_str(&escaped_key);
                text.push_str(seperator);
                text.push_str(&escaped_value);
            }
        } else if property_value.type_ == PropertyType::Property {
            if let Some(children) = &mut property_value.children {
                // adjust value piece of child
                children[2].data = PropertyData::Text(escaped_value.clone());
                children[2].children = None;
                children[2].type_ = PropertyType::Raw;
            }
        } else {
            panic!("Unknown property type: {:?}", property_value.type_);
        }
    }

    pub fn unset(&mut self, key: &str) {
        self.data.remove(key);

        if let Some(index) = self.value_map.get(key) {
            self.values.remove(*index);
            self.value_map.remove(key);
        }
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

        if let PropertyData::Range(range) = &value.data {
            let mut start = range.start;

            if value.children.is_some() {
                for child in value.children.as_ref().unwrap() {
                    let child_range = match &child.data {
                        PropertyData::Range(range) => range,
                        _ => continue,
                    };

                    component.push_str(&iter.get_range(start, child_range.start));

                    if let PropertyType::EscapedValue = child.type_ {
                        let chr = iter.get(child_range.start + 1).unwrap();

                        component.push(match chr {
                            't' => '\t',
                            'r' => '\r',
                            'n' => '\n',
                            'f' => '\x0c',
                            'u' => {
                                let num = u32::from_str_radix(&iter.get_range(
                                    child_range.start + 2,
                                    child_range.start + 6,
                                ), 16).unwrap_or(0);

                                char::from_u32(num).unwrap()
                            },
                            _ => chr,
                        });
                    }

                    start = child_range.end;
                }
            }

            component.push_str(&iter.get_range(start, range.end));
        }

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

    pub fn is_newline_value(value: Option<&PropertyValue>) -> bool {
        if value.is_some() {
            let value = value.unwrap();

            if value.type_ == PropertyType::LineBreak {
                return true;
            } else if value.type_ == PropertyType::Raw {
                return match &value.data {
                    PropertyData::Text(s) => s.trim().is_empty() && s.contains("\n"),
                    _ => false,
                }
            }
        }

        false
    }
}

impl ToString for Properties {
    fn to_string(&self) -> String {
        let mut buf = String::new();
        let mut values = vec![];

        for value in self.values.iter().rev() {
            values.push(value);
        }

        while !values.is_empty() {
            let value = values.pop().unwrap();

            match value.type_ {
                PropertyType::Raw => match &value.data {
                    PropertyData::Text(text) => buf.push_str(text),
                    _ => panic!("Invalid property data for raw property type!"),
                },
                PropertyType::Property => {
                    for child_value in value.children.as_ref().unwrap().iter().rev() {
                        values.push(child_value);
                    }
                },
                _ => if let PropertyData::Range(range) = &value.data {
                    buf.push_str(&self.contents[range.start..range.end]);
                },
            }
        }

        buf
    }
}

#[cfg(test)]
mod test {
    use super::Properties;
    use std::fs::File;
    use std::io;

    fn get_test_props() -> io::Result<Properties> {
        let file = File::open("test.properties").unwrap();
        Properties::new_file(&file)
    }

    #[test]
    fn reads_file() {
        let props = get_test_props();
        assert!(props.is_ok());
    }

    #[test]
    fn simple_parse_check() {
        let props = get_test_props().unwrap();
        assert_eq!(
            props.get("language"),
            Some(&String::from("English")),
        );
    }

    #[test]
    fn complex_parse_check() {
        let props = get_test_props().unwrap();
        assert_eq!(
            props.get("key with spaces"),
            Some(&String::from("This is the value that could be looked up with the key \"key with spaces\".")),
        );
    }

    #[test]
    fn multiline_parse_check() {
        let props = get_test_props().unwrap();
        assert_eq!(
            props.get("message"),
            Some(&String::from("Welcome to Wikipedia!")),
        );
    }

    #[test]
    fn empty_output_check() {
        let props_str = String::new();
        assert_eq!(
            Properties::new_str(&props_str).to_string(),
            props_str,
        )
    }

    #[test]
    fn basic_output_check() {
        let props_str = String::from("simple\\ key = A fun value!\\nWith multiple lines!");
        assert_eq!(
            Properties::new_str(&props_str).to_string(),
            props_str,
        )
    }
}