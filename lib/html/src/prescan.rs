use std::io::Read;

use crate::{io_queue::IoQueue, CharacterEncoding};

/// A data structure for implementing the byte stream pre-scanning algorithm defined in the spec
/// (https://html.spec.whatwg.org/#prescan-a-byte-stream-to-determine-its-encoding)
pub struct HtmlPreScanner<'a, R> {
    byte_stream: &'a IoQueue<R>,
    position: usize,
    max_pos: usize,
}

impl<'a, R: Read> HtmlPreScanner<'a, R> {
    pub fn new(byte_stream: &'a IoQueue<R>) -> Self {
        Self {
            byte_stream,
            position: 0,
            max_pos: 0,
        }
    }

    pub fn pre_scan_byte_stream(mut self) -> Option<CharacterEncoding> {
        if let Some(encoding) = self._pre_scan_byte_stream() {
            return Some(encoding);
        }

        self.get_xml_encoding()
    }

    fn _pre_scan_byte_stream(&mut self) -> Option<CharacterEncoding> {
        // Keep going to the end of the byte stream peek buffer, or until 1024 bytes
        self.max_pos = usize::min(self.byte_stream.peek_len(), 1024) - 1;

        // Step 1: Let fallback encoding be null

        let _fallback_encoding: Option<CharacterEncoding> = None;

        // Step 2: Let position be a pointer to a byte in the input byte stream,
        //         initially pointing at the first byte

        self.position = 0;

        // Step 3: Pre-scan for UTF-16 XML declarations

        if self.contains_bytes(&[0x3C, 0x0, 0x3F, 0x0, 0x78, 0x0])? {
            return Some(CharacterEncoding::Utf16LE);
        }

        if self.contains_bytes(&[0x0, 0x3C, 0x0, 0x3F, 0x0, 0x78])? {
            return Some(CharacterEncoding::Utf16BE);
        }

        // Step 4: Loop
        'next_byte: loop {
            self.assert_pos()?;

            let letters: Vec<u8> = (0x41..0x5A).chain(0x61..0x7A).collect();

            // `<!--`
            if self.contains_bytes(&[0x3C, 0x21, 0x2D, 0x2D])? {
                // `-->`
                while !self.contains_bytes(&[0x2D, 0x2D, 0x3E])? {
                    self.assert_pos()?;

                    self.position += 1;
                }

                // Advance pointer to point to first 0x3E byte
                self.position += 2;
            }
            // case-insensitive ASCII '<meta' followed by a space or slash
            else if self.matches_sequence(&[
                vec![0x3C],
                vec![0x4D, 0x6D],
                vec![0x45, 0x65],
                vec![0x54, 0x74],
                vec![0x41, 0x61],
                vec![0x09, 0x0A, 0x0C, 0x0D, 0x20, 0x2F],
            ])? {
                // Step 1: Advance pointer to point to first whitespace byte
                self.position += 5;

                // Steps 2-5
                let mut attributes: Vec<String> = Vec::new();
                let mut got_pragma = false;
                let mut need_pragma: Option<bool> = None;
                let mut charset: Option<CharacterEncoding> = None;

                // Step 6
                'attributes: loop {
                    // get an attribute algorithm
                    let attribute = self.get_attribute()?;

                    let Some((name, value)) = attribute else {
                        break 'attributes;
                    };

                    // Step 7
                    if attributes.contains(&name) {
                        continue 'attributes;
                    }

                    // Step 8
                    attributes.push(name.clone());

                    // Step 9
                    match name.as_str() {
                        "http-equiv" => {
                            if value == "content-type" {
                                got_pragma = true;
                            }
                        }
                        "content" => {
                            let encoding = Self::extract_encoding_from_meta(value);

                            if let (Some(encoding), None) = (encoding, &charset) {
                                charset = Some(encoding);
                                need_pragma = Some(true);
                            }
                        }
                        "charset" => {
                            charset = value.parse::<CharacterEncoding>().ok();
                            need_pragma = Some(false);
                        }
                        _ => {}
                    }

                    // Step 10
                }

                // Step 11
                let Some(need_pragma) = need_pragma else {
                    self.position += 1;
                    continue 'next_byte;
                };

                // Step 12
                if need_pragma && !got_pragma {
                    self.position += 1;
                    continue 'next_byte;
                }

                // Step 13
                let Some(mut charset) = charset else {
                    self.position += 1;
                    continue 'next_byte;
                };

                // Step 14
                if matches!(
                    charset,
                    CharacterEncoding::Utf16BE | CharacterEncoding::Utf16LE
                ) {
                    charset = CharacterEncoding::Utf8
                }

                // Step 15
                if matches!(charset, CharacterEncoding::XUserDefined) {
                    charset = CharacterEncoding::Windows1252
                }

                // Step 16
                return Some(charset);
            }
            // other tag
            else if self.matches_sequence(&[vec![0x3C], letters.clone()])?
                || self.matches_sequence(&[vec![0x3C], vec![0x2F], letters.clone()])?
            {
                // Step 1
                while !self.matches_sequence(&[vec![0x09, 0x0A, 0x0C, 0x0D, 0x20, 0x3E]])? {
                    self.position += 1;
                }

                self.assert_pos()?;

                // Step 2
                while self.get_attribute()?.is_some() {}
            }
            // special tags
            else if self.matches_sequence(&[vec![0x3C], vec![0x21, 0x2F, 0x3F]])? {
                while !self.contains_bytes(&[0x3E])? {
                    self.position += 1;
                }

                self.assert_pos()?;
            }

            self.position += 1;
        }
    }

    fn extract_encoding_from_meta(value: String) -> Option<CharacterEncoding> {
        // Step 1
        let mut position = 0;

        'outer: loop {
            // Step 2
            while &value[position..position + 7] != "charset" {
                if position > value.len() {
                    return None;
                }

                position += 1;
            }

            position += 7;

            // Step 3
            while value.chars().nth(position).unwrap().is_ascii_whitespace() {
                if position > value.len() {
                    return None;
                }

                position += 1;
            }

            // Step 4
            if value.chars().nth(position).unwrap() != '=' {
                position -= 1;
                continue 'outer;
            }

            position += 1;

            // Step 5
            while value.chars().nth(position).unwrap().is_ascii_whitespace() {
                if position > value.len() {
                    return None;
                }

                position += 1;
            }

            if value.chars().nth(position).unwrap() == '"'
                || value.chars().nth(position).unwrap() == '\''
            {
                let quote = value.chars().nth(position).unwrap();

                if !&value[position..].contains(quote) {
                    return None;
                }

                let last_index = value[position..]
                    .chars()
                    .enumerate()
                    .find(|(_, c)| *c == quote)
                    .map(|(i, _)| i)
                    .unwrap();

                let string = &value[position..last_index];

                return string.parse().ok();
            }

            let last_index = value[position..]
                .chars()
                .enumerate()
                .find(|(_, c)| c.is_ascii_whitespace() || *c == ';')
                .map(|(i, _)| i)
                .unwrap_or_else(|| value.len());

            let string = &value[position..last_index];

            return string.parse().ok();
        }
    }

    fn get_attribute(&mut self) -> Option<Option<(String, String)>> {
        // Skip ascii whitespace
        while self.matches_sequence(&[vec![0x09, 0x0A, 0x0C, 0x0D, 0x20, 0x2F]])? {
            self.position += 1;
        }

        self.assert_pos()?;

        // If the byte at position is 0x3E (>), then abort the get an attribute algorithm
        if self.contains_bytes(&[0x3E])? {
            return Some(None);
        }

        let mut name = String::new();
        let mut value = String::new();

        // Step 4
        'parse_attribute_name: loop {
            self.assert_pos()?;

            match self.current_byte()? {
                // `=`
                0x3D if !name.is_empty() => {
                    self.position += 1;

                    // Jump to Value
                    break 'parse_attribute_name;
                }
                // Space
                0x09 | 0x0A | 0x0C | 0x0D | 0x20 => {
                    // Step 6
                    while self.matches_sequence(&[vec![0x09, 0x0A, 0x0C, 0x0D, 0x20]])? {
                        self.position += 1;
                    }

                    self.assert_pos()?;

                    // Step 7
                    if !self.contains_bytes(&[0x3D])? {
                        return Some(Some((name, value)));
                    }

                    // Step 8
                    self.position += 1;

                    // Jump to Value
                    break 'parse_attribute_name;
                }
                0x2F | 0x3E => return Some(Some((name, value))),
                c @ 0x41..=0x5A => name.push((c + 0x20) as char),
                c => name.push(c as char),
            }

            // Step 5
            self.position += 1;
        }

        self.assert_pos()?;

        // Step 9
        while self.matches_sequence(&[vec![0x09, 0x0A, 0x0C, 0x0D, 0x20]])? {
            self.position += 1;
        }

        self.assert_pos()?;

        // Step 10
        match self.current_byte()? {
            b @ (0x22 | 0x27) => loop {
                self.position += 1;

                self.assert_pos()?;

                match self.current_byte()? {
                    x if x == b => {
                        self.position += 1;
                        return Some(Some((name, value)));
                    }
                    x @ 0x41..=0x5A => value.push((x + 0x20) as char),
                    x => value.push(x as char),
                }
            },
            0x3E => return Some(Some((name, value))),
            x @ 0x41..=0x5A => value.push((x + 0x20) as char),
            x => value.push(x as char),
        }

        self.position += 1;
        self.assert_pos()?;

        // Step 11
        loop {
            match self.current_byte()? {
                0x09 | 0x0A | 0x0C | 0x0D | 0x20 | 0x3E => return Some(Some((name, value))),
                x @ 0x41..=0x5A => value.push((x + 0x20) as char),
                x => value.push(x as char),
            }

            // Step 12
            self.position += 1;
            self.assert_pos()?;
        }
    }

    fn get_xml_encoding(mut self) -> Option<CharacterEncoding> {
        // Step 1
        self.position = 0;

        // Step 2
        if !self.contains_bytes(&[0x3C, 0x3F, 0x78, 0x6D, 0x6C])? {
            return None;
        }

        // Step 3
        let mut found = false;
        for i in self.position..self.max_pos {
            if self.byte_stream.peek_nth(i).unwrap() == 0x3E {
                found = true;
            }
        }

        if !found {
            return None;
        }

        // Step 4
        while !self.contains_bytes(&[0x65, 0x6E, 0x63, 0x6F, 0x64, 0x69, 0x6E, 0x67])? {
            self.position += 1;
        }

        // Step 5
        self.position += 8;

        // Step 6
        while self.current_byte()? <= 0x20 {
            self.position += 1;
        }

        // Step 7
        if self.current_byte()? != 0x3D {
            return None;
        }

        // Step 8
        self.position += 1;

        // Step 9
        while self.current_byte()? <= 0x20 {
            self.position += 1;
        }

        // Step 10
        let quote_mark = self.current_byte()?;

        // Step 11
        if !matches!(quote_mark, 0x22 | 0x27) {
            return None;
        }

        // Step 12
        self.position += 1;

        // Step 13
        let mut encoding_end_position = None;
        for i in self.position..self.max_pos {
            if self.byte_stream.peek_nth(i).unwrap() == 0x3E {
                encoding_end_position = Some(i);
            }
        }

        let Some(encoding_end_position) = encoding_end_position else {
            return None;
        };

        // Step 14
        let mut potential_encoding = Vec::new();
        for i in self.position..encoding_end_position {
            potential_encoding.push(self.byte_stream.peek_nth(i).unwrap())
        }

        // Step 15
        if potential_encoding.iter().any(|b| *b < 0x20) {
            return None;
        }

        // Step 16
        let potential_encoding = String::from_utf8(potential_encoding).ok()?;

        let mut encoding = potential_encoding.parse().ok()?;

        // Step 17
        if matches!(
            encoding,
            CharacterEncoding::Utf16BE | CharacterEncoding::Utf16LE
        ) {
            encoding = CharacterEncoding::Utf8;
        }

        // Step 18
        Some(encoding)
    }

    /* Helper methods for structure */

    fn contains_bytes(&self, bytes: &[u8]) -> Option<bool> {
        if self.position + bytes.len() > self.max_pos {
            return None;
        }

        Some(self.byte_stream.contains_bytes(self.position, bytes))
    }

    fn matches_sequence(&self, sequence: &[Vec<u8>]) -> Option<bool> {
        if self.position + sequence.len() > self.max_pos {
            return None;
        }

        Some(self.byte_stream.matches_sequence(self.position, sequence))
    }

    fn assert_pos(&self) -> Option<()> {
        if self.position > self.max_pos {
            None
        } else {
            Some(())
        }
    }

    fn current_byte(&self) -> Option<u8> {
        self.byte_stream.peek_nth(self.position)
    }
}
