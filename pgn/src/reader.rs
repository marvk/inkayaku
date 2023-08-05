use std::collections::HashMap;
use std::io::Read;

use crate::reader::PgnRawParserError::ReadingFromClosedRead;

#[derive(Debug)]
pub struct PgnRawAnnotatedMove {
    pub mv: String,
    pub annotation: Option<String>,
}

impl PgnRawAnnotatedMove {
    pub const fn new(mv: String, annotation: Option<String>) -> Self {
        Self { mv, annotation }
    }
}

#[derive(Debug)]
pub struct PgnRaw {
    pub tag_pairs: HashMap<String, String>,
    pub moves: Vec<PgnRawAnnotatedMove>,
}

impl PgnRaw {
    pub fn new(tag_pairs: HashMap<String, String>, moves: Vec<PgnRawAnnotatedMove>) -> Self {
        Self { tag_pairs, moves }
    }
}

pub struct PgnRawParser<R: Read> {
    reader: R,
    chunk_size: usize,
    eof_reached: bool,
    current_buffer: Vec<u8>,
    current_byte: usize,
    position: u64,
}

#[derive(Debug)]
pub enum PgnRawParserError {
    ReadingFromClosedRead,
    IllegalConsume { position: u64, expected: u8, actual: u8 },
    IllegalSymbol { position: u64, actual: u8 },
}

impl<R: Read> PgnRawParser<R> {
    pub fn new(reader: R) -> Self {
        Self::with_chunk_size(reader, 8192)
    }

    pub fn with_chunk_size(reader: R, chunk_size: usize) -> Self {
        Self { reader, chunk_size, eof_reached: false, current_buffer: vec![0; chunk_size], current_byte: chunk_size, position: 0 }
    }

    fn ensure_buffer(&mut self) -> bool {
        if self.current_byte >= self.current_buffer.len() {
            self.current_byte = 0;
            let result = self.reader.read(&mut self.current_buffer);
            match result {
                Ok(0) => {
                    self.current_buffer.clear();
                    self.eof_reached = true;
                    return false;
                }
                Ok(bytes_read) if bytes_read < self.chunk_size => {
                    self.current_buffer.resize(bytes_read, 0);
                }
                Ok(bytes_read) if bytes_read > self.chunk_size => {
                    panic!("Assertion Error");
                }
                _ => ()
            };
        }

        true
    }

    fn peek_byte(&mut self) -> Result<u8, PgnRawParserError> {
        if self.ensure_buffer() {
            Ok(self.current_buffer[self.current_byte])
        } else {
            Err(ReadingFromClosedRead)
        }
    }

    fn pop_byte(&mut self) -> Result<u8, PgnRawParserError> {
        let result = self.peek_byte()?;
        self.increment_byte();
        Ok(result)
    }

    fn skip_byte(&mut self) -> Result<(), PgnRawParserError> {
        if self.ensure_buffer() {
            self.increment_byte();
            Ok(())
        } else {
            Err(ReadingFromClosedRead)
        }
    }

    fn increment_byte(&mut self) {
        self.current_byte += 1;
        self.position += 1;
    }

    fn consume(&mut self, expected: u8) -> Result<(), PgnRawParserError> {
        let actual = self.pop_byte()?;
        if actual == expected {
            Ok(())
        } else {
            Err(PgnRawParserError::IllegalConsume { position: self.position, expected, actual })
        }
    }

    fn skip_blank_lines(&mut self) -> Result<(), PgnRawParserError> {
        while self.peek_byte()? == b'\n' {
            self.skip_byte()?;
        }

        Ok(())
    }

    fn skip_blank_lines_and_spaces(&mut self) -> Result<(), PgnRawParserError> {
        while self.peek_byte()? == b'\n' || self.peek_byte()? == b' ' {
            self.skip_byte()?;
        }

        Ok(())
    }

    fn skip_spaces(&mut self) -> Result<(), PgnRawParserError> {
        while self.peek_byte()? == b' ' {
            self.skip_byte()?;
        }

        Ok(())
    }

    fn skip_to_next_line(&mut self) -> Result<(), PgnRawParserError> {
        while self.pop_byte()? != b'\n' {};

        Ok(())
    }

    fn read_until(&mut self, byte: u8) -> Result<String, PgnRawParserError> {
        let mut result = String::new();
        let mut cur_byte = self.peek_byte()?;

        while cur_byte != byte {
            result.push(cur_byte as char);
            self.skip_byte()?;
            cur_byte = self.peek_byte()?;
        }

        Ok(result)
    }

    fn read_tag_pairs(&mut self) -> Result<HashMap<String, String>, PgnRawParserError> {
        let mut result = HashMap::new();

        loop {
            match self.peek_byte()? {
                b'[' => {
                    let (k, v) = self.read_tag_pair_line()?;
                    result.insert(k, v);
                }
                b'\n' => { return Ok(result); }
                other => { return Err(PgnRawParserError::IllegalSymbol { position: self.position, actual: other }); }
            }
        }
    }

    fn read_tag_pair_line(&mut self) -> Result<(String, String), PgnRawParserError> {
        self.consume(b'[')?;
        let name = self.read_tag_name()?;
        self.consume(b' ')?;
        let value = self.read_tag_value()?;
        self.consume(b']')?;
        self.consume(b'\n')?;
        Ok((name, value))
    }

    fn read_tag_name(&mut self) -> Result<String, PgnRawParserError> {
        self.read_until(b' ')
    }

    fn read_tag_value(&mut self) -> Result<String, PgnRawParserError> {
        self.consume(b'"')?;
        let value = self.read_until(b'"');
        self.consume(b'"')?;
        value
    }

    fn read_moves(&mut self) -> Result<Vec<PgnRawAnnotatedMove>, PgnRawParserError> {
        let mut result = Vec::new();

        while let Some(mv) = self.read_move()? {
            result.push(mv);
        }

        self.skip_to_next_line()?;

        Ok(result)
    }

    fn read_move(&mut self) -> Result<Option<PgnRawAnnotatedMove>, PgnRawParserError> {
        self.skip_blank_lines_and_spaces()?;

        let token = self.read_until(b' ')?;

        let mut chars = token.chars();
        if chars.next() == Some('*') {
            return Ok(None);
        }

        if let Some('-' | '/') = chars.next() {
            self.skip_to_next_line()?;
            return Ok(None);
        }

        let mv = if token.contains('.') {
            self.skip_spaces()?;
            self.read_until(b' ')?
        } else {
            token
        };

        self.skip_spaces()?;

        let byte = self.peek_byte()?;

        let annotation = match byte {
            b'{' => Some(self.read_braced_annotation()?),
            b';' => Some(self.read_semicolon_annotation()?),
            _ => None,
        };

        Ok(Some(PgnRawAnnotatedMove::new(mv, annotation)))
    }

    fn read_braced_annotation(&mut self) -> Result<String, PgnRawParserError> {
        self.consume(b'{')?;
        let result = self.read_until(b'}');
        self.consume(b'}')?;
        result
    }

    fn read_semicolon_annotation(&mut self) -> Result<String, PgnRawParserError> {
        self.consume(b';')?;
        let result = self.read_until(b'\n');
        self.consume(b'\n')?;
        result
    }

    fn read_pgn(&mut self) -> Result<PgnRaw, PgnRawParserError> {
        let tag_pairs = self.read_tag_pairs()?;

        self.skip_blank_lines()?;

        let moves = self.read_moves()?;


        let raw = PgnRaw::new(tag_pairs, moves);
        Ok(raw)
    }
}

impl<R: Read> Iterator for PgnRawParser<R> {
    type Item = Result<PgnRaw, PgnRawParserError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.skip_blank_lines_and_spaces() {
            Ok(()) => { Some(self.read_pgn()) }
            Err(ReadingFromClosedRead) => { None }
            Err(err) => { Some(Err(err)) }
        }
    }
}
