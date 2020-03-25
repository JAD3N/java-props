use std::collections::HashMap;

lazy_static! {
    static ref KEY_ESCAPE_MAP: HashMap<char, &'static str> = {
        let mut m = HashMap::new();
        m.insert(' ', "\\ ");
        m.insert('\n', "\\n");
        m.insert(':', "\\:");
        m.insert('=', "\\=");
        m
    };

    static ref VALUE_ESCAPE_MAP: HashMap<char, &'static str> = {
        let mut m = HashMap::new();
        m.insert('\n', "\\n");
        m
    };
}

pub fn escape(s: &str, map: &HashMap<char, &str>) -> String {
    let mut buf = String::new();
    let zeros = [ "", "0", "00", "000" ];

    for chr in s.chars() {
        if map.contains_key(&chr) {
            buf.push_str(map.get(&chr).unwrap());
        } else {
            let code = chr as u32;

            if code <= 0x7F {
                buf.push(chr);
            } else {
                let hex = format!("{:x}", code);

                buf.push_str("\\u");

                if hex.len() < 4 {
                    buf.push_str(zeros[4 - hex.len()]);
                }

                buf.push_str(&hex);
            }
        }
    }

    buf
}

pub fn escape_key(s: &str) -> String {
    escape(s, &(*KEY_ESCAPE_MAP))
}

pub fn escape_value(s: &str) -> String {
    escape(s, &(*VALUE_ESCAPE_MAP))
}