pub struct Iterator {
    pub position: usize,
    length: usize,
    chars: Vec<char>,
}

impl Iterator {
    pub fn new(s: &String) -> Iterator {
        let chars: Vec<char> = s.chars().collect();

        Iterator {
            position: 0,
            length: chars.len(),
            chars,
        }
    }

    pub fn peek_x(&self, i: usize) -> Option<char> {
        if self.position + i >= self.length {
            None
        } else {
            Some(self.chars[self.position + i])
        }
    }

    pub fn peek(&self) -> Option<char> {
        self.peek_x(0)
    }

    pub fn next_x(&mut self, i: usize) -> Option<char> {
        if self.position >= self.length {
            None
        } else {
            let pos = self.position;
            self.position += i;
            Some(self.chars[pos])
        }
    }

    pub fn next(&mut self) -> Option<char> {
        self.next_x(1)
    }

    pub fn get(&self, i: usize) -> Option<char> {
        if i >= self.length {
            None
        } else {
            Some(self.chars[i])
        }
    }

    pub fn get_range(&self, start: usize, end: usize) -> String {
        let mut s = String::new();

        for i in start..end {
            s.push(self.get(i).unwrap());
        }

        s
    }
}