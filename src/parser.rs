use crate::{PropertyValue, PropertyRange, PropertyData, PropertyType::*};
use crate::Iterator;

pub fn read_whitespace(iter: &mut Iterator) -> PropertyValue {
    let start = iter.position;

    while match iter.peek() {
        Some(chr) => chr.is_whitespace(),
        None => false,
    } {
        iter.next();
    }

    PropertyValue {
        data: PropertyData::Range(PropertyRange {
            start,
            end: iter.position,
        }),
        children: None,
        type_: Whitespace,
    }
}

pub fn is_comment_indicator(chr: char) -> bool {
    chr == '#' || chr == '!'
}

pub fn is_eol(chr: char) -> bool {
    chr == '\n' || chr == '\r'
}

pub fn read_comment(iter: &mut Iterator) -> PropertyValue {
    let start = iter.position;

    while match iter.peek() {
        Some(chr) => !is_eol(chr),
        None => false
    } {
        iter.next();
    }

    PropertyValue {
        data: PropertyData::Range(PropertyRange {
            start,
            end: iter.position,
        }),
        children: None,
        type_: Comment,
    }
}

pub fn starts_escaped_value(chr: char) -> bool {
    chr == '\\'
}

pub fn read_escaped_value(iter: &mut Iterator) -> PropertyValue {
    let start = iter.position;

    // skip "\"
    iter.next();

    let chr = iter.next();
    if chr.is_some() {
        let chr = chr.unwrap();

        if chr == 'u' {
            iter.next_x(4);
        }
    }

    PropertyValue {
        data: PropertyData::Range(PropertyRange {
            start,
            end: iter.position,
        }),
        children: None,
        type_: EscapedValue,
    }
}

pub fn starts_separator(chr: char) -> bool {
    chr == '=' || chr == ':' || chr.is_whitespace()
}

pub fn read_key(iter: &mut Iterator) -> PropertyValue {
    let start = iter.position;
    let mut children = Vec::new();

    while iter.peek().is_some() {
        let chr = iter.peek().unwrap();

        if starts_separator(chr) {
            break;
        }

        if starts_escaped_value(chr) {
            children.push(read_escaped_value(iter));
            continue;
        }

        iter.next();
    }

    PropertyValue {
        data: PropertyData::Range(PropertyRange {
            start,
            end: iter.position,
        }),
        children: Some(children),
        type_: Key,
    }
}

pub fn read_property(iter: &mut Iterator) -> PropertyValue {
    let start = iter.position;
    let children = vec![
        read_key(iter),
        read_separator(iter),
        read_value(iter),
    ];

    PropertyValue {
        data: PropertyData::Range(PropertyRange {
            start,
            end: iter.position,
        }),
        children: Some(children),
        type_: Property,
    }
}

pub fn read_separator(iter: &mut Iterator) -> PropertyValue {
    let start = iter.position;
    let mut after_separator = false;

    while iter.peek().is_some() {
        let chr = iter.peek().unwrap();

        if is_eol(chr) {
            break;
        }

        if chr.is_whitespace() {
            iter.next();
            continue;
        }

        if after_separator {
            break;
        }

        after_separator = chr == ':' || chr == '=';

        if after_separator {
            iter.next();
            continue;
        }

        break;
    }

    PropertyValue {
        data: PropertyData::Range(PropertyRange {
            start,
            end: iter.position,
        }),
        children: None,
        type_: Separator,
    }
}

pub fn starts_line_break(iter: &mut Iterator) -> bool {
    iter.peek().unwrap() == '\\' && is_eol(iter.peek_x(1).unwrap())
}

pub fn read_line_break(iter: &mut Iterator) -> PropertyValue {
    let start = iter.position;

    iter.next();

    if let Some(chr) = iter.peek() {
        if chr == '\r' {
            iter.next();
        }
    }

    iter.next();

    while iter.peek().is_some() {
        let chr = iter.peek().unwrap();

        if is_eol(chr) || !chr.is_whitespace() {
            break;
        }

        iter.next();
    }

    PropertyValue {
        data: PropertyData::Range(PropertyRange {
            start,
            end: iter.position,
        }),
        children: None,
        type_: LineBreak,
    }
}

pub fn read_value(iter: &mut Iterator) -> PropertyValue {
    let start = iter.position;
    let mut children = Vec::new();

    while iter.peek().is_some() {
        let chr = iter.peek().unwrap();

        if starts_line_break(iter) {
            children.push(read_line_break(iter));
            continue;
        }

        if starts_escaped_value(chr) {
            children.push(read_escaped_value(iter));
            continue;
        }

        if is_eol(chr) {
            break;
        }

        iter.next();
    }

    PropertyValue {
        data: PropertyData::Range(PropertyRange {
            start,
            end: iter.position,
        }),
        children: Some(children),
        type_: Value,
    }
}
