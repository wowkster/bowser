use std::{
    cell::RefCell,
    collections::VecDeque,
    io::{BufReader, Read},
};

pub struct IoQueue<R> {
    stream: RefCell<BufReader<R>>,
    peeked: RefCell<VecDeque<u8>>,
    bytes_read: RefCell<usize>,
}

impl<R: Read> IoQueue<R> {
    pub fn new(stream: R) -> Self {
        Self {
            stream: RefCell::new(BufReader::new(stream)),
            peeked: RefCell::new(VecDeque::new()),
            bytes_read: RefCell::new(0),
        }
    }

    pub fn bytes_read(&self) -> usize {
        *self.bytes_read.borrow()
    }

    pub fn next_byte(&mut self) -> Option<u8> {
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

        *self.bytes_read.borrow_mut() += 1;

        Some(buf[0])
    }

    pub fn peek(&self) -> Option<u8> {
        let mut peeked = self.peeked.borrow_mut();

        if !peeked.is_empty() {
            return peeked.front().copied();
        }

        let mut stream = self.stream.borrow_mut();

        let mut buf = vec![0; 1];
        let bytes_read = stream
            .read(&mut buf)
            .expect("Could not read from byte stream");

        if bytes_read == 0 {
            return None;
        }

        *self.bytes_read.borrow_mut() += 1;

        peeked.push_back(buf[0]);

        peeked.front().copied()
    }

    pub fn peek_nth(&self, n: usize) -> Option<u8> {
        let mut peeked = self.peeked.borrow_mut();

        if peeked.len() > n {
            return peeked.get(n).copied();
        }

        let mut stream = self.stream.borrow_mut();

        let chars_to_peek = n + 1 - peeked.len();

        let mut buf = vec![0; chars_to_peek];
        let bytes_read = stream
            .read(&mut buf)
            .expect("Could not read from byte stream");

        buf.iter()
            .take(bytes_read)
            .for_each(|b| peeked.push_back(*b));

        if bytes_read < chars_to_peek {
            return None;
        }

        *self.bytes_read.borrow_mut() += bytes_read;

        return peeked.get(n).copied();
    }

    pub fn peek_arr(&self, n: usize) -> Vec<u8> {
        let mut res = Vec::with_capacity(n);
        let mut i = 0;

        while let Some(b) = self.peek_nth(i) {
            res.push(b);

            if i >= n {
                break;
            }

            i += 1;
        }

        res
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
            if self.peek_nth(n).is_none() {
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
        self.next_byte()
    }
}
