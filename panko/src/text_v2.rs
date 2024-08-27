use core::str::CharIndices;

pub trait BoundedLinesV2<'a, F>
where
    Self: 'a,
    F: FnMut(char) -> u32,
{
    fn bounded_lines_v2(&self, max_width: u32, get_width: F) -> BoundedLinesIter<'a, F>;
}

impl<'a, F> BoundedLinesV2<'a, F> for &'a str where F: FnMut(char) -> u32 {
    fn bounded_lines_v2(&self, max_width: u32, get_width: F) -> BoundedLinesIter<'a, F> {
        let iter = self.char_indices();
        BoundedLinesIter {
            text: self,
            iter,
            max_width,
            char_width: get_width,
            word_start: 0,
            line_start: 0,
            line_end: 0,
            line_width: 0,
            finished: false,
        }
    }
}

pub struct BoundedLinesIter<'a, F>
where
    F: FnMut(char) -> u32,
{
    text: &'a str,
    iter: CharIndices<'a>,
    max_width: u32,
    char_width: F,
    word_start: usize,
    line_start: usize,
    line_end: usize,
    line_width: u32,
    finished: bool,
}

impl<'a, F> Iterator for BoundedLinesIter<'a, F>
where
    F: FnMut(char) -> u32,
{
    type Item = (&'a str, u32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        loop {
            match self.iter.next() {
                Some((i, c)) => {
                    if c != ' ' || i == 0 {
                        continue;
                    }
                    let word = &self.text[self.word_start..i];
                    let word_width: u32 = word.chars().map(&mut self.char_width).sum();
                    let will_overflow = self.line_width + word_width >= self.max_width;
                    if will_overflow {
                        let line = &self.text[self.line_start..self.line_end];
                        let width = self.line_width;
                        self.line_start = self.word_start;
                        self.line_end = i;
                        self.line_width = word_width;
                        self.word_start = i;
                        return Some((line, width));
                    } else {
                        let whitespace_width = (self.char_width)(' ');
                        self.line_width += word_width + whitespace_width;
                        self.line_end = i;
                        self.word_start = i + 1;
                    }
                }
                None => {
                    let line = &self.text[self.line_start..];
                    let width = self.line_width;
                    self.finished = true;
                    return Some((line, width));
                }
            }
        }
    }
}
