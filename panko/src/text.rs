use std::str::CharIndices;

pub trait BoundedLines<'a, F>
where
    Self: 'a,
    F: FnMut(char) -> u32,
{
    fn bounded_lines(&self, max_width: u32, get_width: F) -> BoundedLinesIter<'a, F>;
}

impl<'a, F> BoundedLines<'a, F> for &'a str where F: FnMut(char) -> u32 {
    fn bounded_lines(&self, max_width: u32, get_width: F) -> BoundedLinesIter<'a, F> {
        let words_iter = self.char_indices().word_or_whitespaces();
        BoundedLinesIter {
            text: self,
            words_iter,
            get_width,
            max_width,
            line_start: 0,
            line_end: 0,
            line_width: 0,
            finished: false,
        }
    }
}

pub struct BoundedLinesIter<'a, F: FnMut(char) -> u32> {
    text: &'a str,
    words_iter: WordOrWhitespaceIter<'a>,
    get_width: F,
    max_width: u32,
    line_start: usize,
    line_end: usize,
    line_width: u32,
    finished: bool,
}

pub struct BoundedLine<'a> {
    pub line: &'a str,
    pub width: u32,
}

impl<'a, F> Iterator for BoundedLinesIter<'a, F>
where
    F: FnMut(char) -> u32,
{
    type Item = BoundedLine<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        loop {
            match self.words_iter.next() {
                Some(WordOrWhitespace::Word {
                    start,
                    end,
                    inclusive,
                }) => {
                    let word = if inclusive {
                        &self.text[start..=end]
                    } else {
                        &self.text[start..end]
                    };
                    let width = word.chars().map(&mut self.get_width).sum::<u32>();
                    if self.line_width + width > self.max_width {
                        let line = &self.text[self.line_start..=self.line_end];
                        let line_width = self.line_width;
                        self.line_start = start;
                        self.line_end = end;
                        self.line_width = width;
                        return Some(BoundedLine { line, width: line_width });
                    } else {
                        self.line_end = end;
                        self.line_width += width;
                    }
                }
                Some(WordOrWhitespace::Whitespace { index }) => {
                    let whitespace_width = (self.get_width)(' ');
                    if self.line_width + whitespace_width > self.max_width {
                        let line = &self.text[self.line_start..=self.line_end];
                        let line_width = self.line_width;
                        self.line_start = index + 1;
                        self.line_end = index + 1;
                        self.line_width = 0;
                        return Some(BoundedLine {
                            line,
                            width: line_width,
                        });
                    } else {
                        self.line_end = index;
                        self.line_width += whitespace_width;
                    }
                }
                None => {
                    self.finished = true;
                    return Some(BoundedLine {
                        line: &self.text[self.line_start..=self.line_end],
                        width: self.line_width,
                    });
                }
            }
        }
    }
}

/// An iterator over Words or Whitespaces blocks.
pub trait WordOrWhitespaces<'a> {
    fn word_or_whitespaces(self) -> WordOrWhitespaceIter<'a>;
}

impl<'a> WordOrWhitespaces<'a> for CharIndices<'a> {
    fn word_or_whitespaces(self) -> WordOrWhitespaceIter<'a> {
        WordOrWhitespaceIter {
            iter: self,
            state: WordOrWhitespaceIterState::Begin,
            previous_char_index: 0,
        }
    }
}

pub struct WordOrWhitespaceIter<'a> {
    iter: CharIndices<'a>,
    state: WordOrWhitespaceIterState,
    previous_char_index: usize,
}

pub enum WordOrWhitespace {
    Word {
        start: usize,
        end: usize,
        inclusive: bool,
    },
    Whitespace {
        index: usize,
    },
}

enum WordOrWhitespaceIterState {
    Begin,
    Word { start: usize },
    PendingWhitespace { index: usize },
    Whitespaces,
    Finished,
}

impl<'a> Iterator for WordOrWhitespaceIter<'a> {
    type Item = WordOrWhitespace;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.state {
                WordOrWhitespaceIterState::Begin => match self.iter.next() {
                    Some((i, c)) => {
                        self.previous_char_index = i;
                        if c == ' ' {
                            self.state = WordOrWhitespaceIterState::Whitespaces;
                            return Some(WordOrWhitespace::Whitespace { index: i });
                        } else {
                            self.state = WordOrWhitespaceIterState::Word { start: i };
                        }
                    }
                    None => self.state = WordOrWhitespaceIterState::Finished,
                },
                WordOrWhitespaceIterState::Word { start } => match self.iter.next() {
                    Some((i, c)) => {
                        self.previous_char_index = i;
                        if c == ' ' {
                            self.state = WordOrWhitespaceIterState::PendingWhitespace { index: i };
                            return Some(WordOrWhitespace::Word {
                                start,
                                end: i,
                                inclusive: false,
                            });
                        }
                    }
                    None => {
                        self.state = WordOrWhitespaceIterState::Finished;
                        return Some(WordOrWhitespace::Word {
                            start,
                            end: self.previous_char_index,
                            inclusive: true,
                        });
                    }
                },
                WordOrWhitespaceIterState::PendingWhitespace { index } => {
                    self.state = WordOrWhitespaceIterState::Whitespaces;
                    return Some(WordOrWhitespace::Whitespace { index });
                }
                WordOrWhitespaceIterState::Whitespaces => match self.iter.next() {
                    Some((i, c)) => {
                        self.previous_char_index = i;
                        if c == ' ' {
                            return Some(WordOrWhitespace::Whitespace { index: i });
                        } else {
                            self.state = WordOrWhitespaceIterState::Word { start: i };
                        }
                    }
                    None => {
                        self.state = WordOrWhitespaceIterState::Finished;
                    }
                },
                WordOrWhitespaceIterState::Finished => return None,
            }
        }
    }
}
