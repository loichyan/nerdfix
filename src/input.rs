use std::collections::VecDeque;
use std::io::BufRead;
use std::{io, iter};

use content_inspector::ContentType;
use miette::{MietteSpanContents, SourceCode, SourceSpan, SpanContents};

const LINES_BEFORE: usize = 1;
const LINES_AFTER: usize = 3;

pub struct InputReader<R = Box<dyn BufRead>> {
    reader: R,
    buffer: Vec<u8>,
    /// The absolute positions of each line in the buffer.
    line_sizes: VecDeque<usize>,
    /// The absolute line number of the current line.
    line_count: usize,
    /// The absolute position of the current line.
    offset: usize,
    /// The position of the current line relative to the buffer beginning.
    rel_offset: usize,
}

impl<R: BufRead> InputReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buffer: Vec::new(),
            line_sizes: VecDeque::new(),
            line_count: 0,
            offset: 0,
            rel_offset: 0,
        }
    }

    pub fn next_line(&mut self) -> io::Result<Option<InputLine>> {
        if let Some(n) = self.line_sizes.pop_front() {
            // Shift to the second line
            debug_assert!(self.line_sizes.len() >= LINES_BEFORE);
            self.buffer.drain(..n);
            self.rel_offset -= n;

            // Peek one line ahead
            self.read_line()?;
        } else {
            // In the initial call, pad preceding empty lines,
            self.line_sizes.reserve(LINES_BEFORE + 1 + LINES_AFTER);
            self.line_sizes.extend(iter::repeat(0).take(LINES_BEFORE));

            // and then peek subsequent context lines
            for _ in 0..=LINES_AFTER {
                self.read_line()?;
            }
        }

        let source;
        if let Some(&size) = self.line_sizes.get(LINES_BEFORE) {
            source = Some(InputLine {
                buffer: &self.buffer,
                line_sizes: &self.line_sizes,
                line_count: self.line_count,
                offset: self.offset,
                rel_offset: self.rel_offset,
                size,
            });
            self.line_count += 1;
            self.offset += size;
            self.rel_offset += size;
        } else {
            // EOF reached
            self.line_count = usize::MAX;
            source = None;
        }

        Ok(source)
    }

    fn read_line(&mut self) -> io::Result<usize> {
        // TODO: limit line size
        let size = self.reader.read_until(b'\n', &mut self.buffer)?;
        if size != 0 {
            self.line_sizes.push_back(size);
        }
        Ok(size)
    }
}

#[derive(Debug)]
pub struct InputLine<'a> {
    buffer: &'a [u8],
    line_sizes: &'a VecDeque<usize>,
    line_count: usize,
    offset: usize,
    rel_offset: usize,
    size: usize,
}

impl<'a> InputLine<'a> {
    /// Returns the content of this line.
    pub fn contents(&self) -> &'a [u8] {
        &self.buffer[self.rel_offset..self.rel_offset + self.size]
    }

    /// Returns the absolute offset of a byte index relative to the line start.
    pub fn offset_of(&self, i: usize) -> usize {
        self.offset + i
    }

    pub fn content_type(&self) -> ContentType {
        content_inspector::inspect(self.buffer)
    }
}

impl SourceCode for InputLine<'_> {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        lines_before: usize,
        lines_after: usize,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, miette::MietteError> {
        debug_assert!((self.offset..self.offset + self.size).contains(&span.offset()));

        let start;
        let offset;
        let line;
        let column;
        if lines_before == 0 {
            offset = span.offset();
            column = offset - self.offset;
            start = self.rel_offset + column;
            line = self.line_count;
        } else {
            // count precedent lines and bytes
            let (lines, bytes) = self
                .line_sizes
                .range(0..LINES_BEFORE)
                .copied()
                .rev()
                .take(lines_before)
                .take_while(|&n| n > 0)
                .fold((0, 0), |(lines, bytes), n| (lines + 1, bytes + n));

            offset = self.offset - bytes;
            column = 0;
            start = self.rel_offset - bytes;
            line = self.line_count - lines;
        }

        let end;
        let line_count;
        if lines_after == 0 {
            end = start + span.len();
            line_count = self.line_count;
        } else {
            // count subsequent lines and bytes
            let (lines, bytes) = self
                .line_sizes
                .range(LINES_BEFORE..)
                .copied()
                .take(lines_before + 1)
                .take_while(|&n| n > 0)
                .fold((0, 0), |(lines, bytes), n| (lines + 1, bytes + n));

            end = self.rel_offset + bytes;
            line_count = self.line_count + lines;
        }

        let data = &self.buffer[start..end];
        let span = SourceSpan::from((offset, end - start));

        Ok(Box::new(MietteSpanContents::new(
            data, span, line, column, line_count,
        )))
    }
}
