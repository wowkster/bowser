use std::{
    cell::RefCell,
    collections::VecDeque,
    io::{BufReader, Read},
};

pub struct IoQueue<R> {
    stream: RefCell<BufReader<R>>,
    peeked: RefCell<VecDeque<u8>>,
}

impl<R: Read> IoQueue<R> {
    pub fn new(stream: R) -> Self {
        Self {
            stream: RefCell::new(BufReader::new(stream)),
            peeked: RefCell::new(VecDeque::new()),
        }
    }

    pub fn next(&mut self) -> Option<u8> {
        let mut peeked = self.peeked.borrow_mut();

        if !peeked.is_empty() {
            return peeked.pop_front();
        }

        let mut stream = self.stream.borrow_mut();

        let mut buf = vec![0; 1];
        let bytes_read = stream
            .read(&mut buf)
            .expect("Could not read from byte stream");

        if bytes_read == 0 {
            return None;
        }

        Some(buf[0])
    }

    pub fn peek(&self) -> Option<u8> {
        let mut peeked = self.peeked.borrow_mut();

        if !peeked.is_empty() {
            return peeked.front().map(|b| *b);
        }

        let mut stream = self.stream.borrow_mut();

        let mut buf = vec![0; 1];
        let bytes_read = stream
            .read(&mut buf)
            .expect("Could not read from byte stream");

        if bytes_read == 0 {
            return None;
        }

        peeked.push_back(buf[0]);

        peeked.front().map(|b| *b)
    }

    pub fn peek_nth(&self, n: usize) -> Option<u8> {
        let mut peeked = self.peeked.borrow_mut();

        if peeked.len() > n {
            return peeked.get(n).map(|b| *b);
        }

        let mut stream = self.stream.borrow_mut();

        let chars_to_peek = n + 1 - peeked.len();

        let mut buf = vec![0; chars_to_peek];
        let bytes_read = stream
            .read(&mut buf)
            .expect("Could not read from byte stream");

        for i in 0..bytes_read {
            peeked.push_back(buf[i])
        }

        if bytes_read < chars_to_peek {
            return None;
        }

        return peeked.get(n).map(|b| *b);
    }

    pub fn has_next(&self) -> bool {
        self.peek().is_some()
    }

    pub fn has_next_nth(&self, n: usize) -> bool {
        self.peek_nth(n).is_some()
    }

    /// Reads bytes into the peek buffer while it contains less than `max` bytes
    pub fn peek_max(&self, max: usize) {
        // TODO: optimize
        for n in 0..max {
            if let None = self.peek_nth(n) {
                return;
            }
        }
    }

    pub fn peek_len(&self) -> usize {
        self.peeked.borrow().len()
    }

    pub fn contains_bytes(&self, start_pos: usize, bytes: &[u8]) -> bool {
        for (i, byte) in bytes.iter().enumerate() {
            let Some(b) = self.peek_nth(start_pos + i) else {
                return false;
            };

            if b != *byte {
                return false;
            }
        }

        true
    }

    pub fn matches_sequence(&self, start_pos: usize, sequence: &[Vec<u8>]) -> bool {
        for (i, possible_bytes) in sequence.iter().enumerate() {
            let Some(byte) = self.peek_nth(start_pos + i) else {
                return false;
            };

            if !possible_bytes.contains(&byte) {
                return false;
            }
        }

        true
    }
}

impl<R: Read> Iterator for IoQueue<R> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}
