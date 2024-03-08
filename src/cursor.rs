#[derive(Debug, Clone)]
pub struct CodeCursorPoint {
    pub value: char,
    pub line: usize,
    pub line_idx: usize,
    pub file_idx: usize,
}

#[derive(Debug)]
pub struct CodeCursor {
    inner: Vec<CodeCursorPoint>,
    read_idx: usize,
}

impl CodeCursor {
    pub fn new<S: AsRef<str>>(str: S) -> Self {
        let str = str.as_ref();

        let mut points = Vec::with_capacity(str.len());
        let mut line = 0;
        let mut line_idx = 0;
        let mut file_idx = 0;

        for c in str.chars() {
            points.push(CodeCursorPoint {
                value: c,
                line,
                line_idx,
                file_idx,
            });

            if c == '\n' {
                line += 1;
                line_idx = 0;
            } else {
                line_idx += 1;
            }

            file_idx += c.len_utf8();
        }

        Self {
            inner: points,
            read_idx: 0,
        }
    }

    pub fn is_eof(&self) -> bool {
        self.read_idx >= self.inner.len()
    }

    pub fn peek(&self) -> Option<&CodeCursorPoint> {
        self.inner.get(self.read_idx)
    }

    pub fn peek_n(&self, offset: usize) -> Option<&CodeCursorPoint> {
        self.inner.get(self.read_idx + offset)
    }

    pub fn peek_chr(&self) -> Option<char> {
        self.peek().map(|v| v.value)
    }

    pub fn peek_chr_n(&self, offset: usize) -> Option<char> {
        self.peek_n(offset).map(|v| v.value)
    }

    pub fn pop(&mut self) -> Option<CodeCursorPoint> {
        if self.is_eof() {
            return None;
        }

        let point = self.inner.get(self.read_idx).cloned();
        self.read_idx += 1;
        point
    }

    pub fn pop_if(&mut self, c: char) -> Option<CodeCursorPoint> {
        match self.peek_chr() {
            Some(v) if v == c => self.pop(),
            _ => None,
        }
    }

    /// Continues to remove and collect `CodeCursorPoint`s until a specified character is encountered or EOF is reached.
    ///
    /// # Arguments
    ///
    /// * `c` - The character to stop popping at.
    ///
    /// # Returns
    ///
    /// An `Option` containing a `Vec<CodeCursorPoint>` of all `CodeCursorPoint`s removed up to but not including the specified character.
    ///
    /// Returns `None` if EOF is reached before finding the character.
    pub fn pop_until_chr(&mut self, c: char) -> Option<Vec<CodeCursorPoint>> {
        let mut points = vec![];

        while let Some(chr) = self.peek_chr() {
            if c == chr {
                self.pop();
                return Some(points);
            }

            points.push(self.pop()?);
        }

        None
    }

    pub fn pop_until_chr_or_break(&mut self, c: char) -> Option<Vec<CodeCursorPoint>> {
        let mut points = vec![];

        while let Some(chr) = self.peek_chr() {
            if chr == '\n' {
                return None;
            }

            if c == chr {
                self.pop();
                return Some(points);
            }

            points.push(self.pop()?);
        }

        None
    }

    pub fn pop_while<F: Fn(char) -> bool>(&mut self, predicate: F) -> Vec<CodeCursorPoint> {
        let mut points = vec![];
        while let Some(chr) = self.peek_chr() {
            if !predicate(chr) {
                break;
            }
            points.push(self.pop().expect("unreachable."));
        }
        points
    }

    pub fn skip_while<F: Fn(char) -> bool>(&mut self, predicate: F) {
        while let Some(chr) = self.peek_chr() {
            if !predicate(chr) {
                break;
            }
            self.pop();
        }
    }

    pub fn skip_whitespace(&mut self) {
        self.skip_while(char::is_whitespace)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pop_until_chr_found() {
        let mut cursor = CodeCursor::new("abc\ndef");
        let until_c = cursor.pop_until_chr('c').unwrap();
        assert_eq!(until_c.len(), 2);
        assert_eq!(until_c[1].value, 'b');
        assert_eq!(cursor.peek_chr(), Some('\n'));
    }

    #[test]
    fn test_pop_until_chr_not_found() {
        let mut cursor = CodeCursor::new("abc\ndef");
        assert!(cursor.pop_until_chr('x').is_none());
        assert!(cursor.is_eof());
    }

    #[test]
    fn test_skip_whitespace() {
        let mut cursor = CodeCursor::new("   abc");
        cursor.skip_whitespace();
        assert_eq!(cursor.peek_chr(), Some('a'));
    }

    #[test]
    fn test_skip_while_predicate() {
        let mut cursor = CodeCursor::new("123abc");
        cursor.skip_while(char::is_numeric);
        assert_eq!(cursor.peek_chr(), Some('a'));
    }

    #[test]
    fn test_is_eof() {
        let cursor = CodeCursor::new("");
        assert!(cursor.is_eof());
    }

    #[test]
    fn test_peek_and_pop() {
        let mut cursor = CodeCursor::new("a");
        assert_eq!(cursor.peek_chr(), Some('a'));
        assert_eq!(cursor.pop().unwrap().value, 'a');
        assert!(cursor.is_eof());
    }

    #[test]
    fn test_pop_if() {
        let mut cursor = CodeCursor::new("abc");
        assert_eq!(cursor.pop_if('a').unwrap().value, 'a');
        assert!(cursor.pop_if('a').is_none());
    }
}
