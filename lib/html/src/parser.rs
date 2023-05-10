use std::io::Read;

use crate::{
    character_encoding::CharacterEncoding, io_queue::IoQueue, prescan::HtmlPreScanner, Decoder,
    DecodingError, HtmlParseError, HtmlParseResult,
};

pub struct HtmlParser<R> {
    character_encoding: CharacterEncoding,
    encoding_confidence: EncodingConfidence,
    input_byte_stream: IoQueue<R>,
    read_bytes: Vec<u8>,
}

/// https://html.spec.whatwg.org/#concept-encoding-confidence
#[allow(unused)]
#[derive(Debug, PartialEq, Eq)]
enum EncodingConfidence {
    Tentative,
    Certain,
    Irrelevant,
}

impl<R: Read> HtmlParser<R> {
    pub fn new(input_byte_stream: R) -> Self {
        Self {
            character_encoding: CharacterEncoding::default(),
            encoding_confidence: EncodingConfidence::Tentative,
            input_byte_stream: IoQueue::new(input_byte_stream),
            read_bytes: Vec::new(),
        }
    }

    /// https://html.spec.whatwg.org/#parsing-with-a-known-character-encoding
    pub fn with_definite_encoding(
        input_byte_stream: R,
        character_encoding: CharacterEncoding,
    ) -> Self {
        Self {
            character_encoding,
            encoding_confidence: EncodingConfidence::Certain,
            input_byte_stream: IoQueue::new(input_byte_stream),
            read_bytes: Vec::new(),
        }
    }

    /// Will try to parse an HTML document, but will abort if any error condition is discovered.
    /// This behavior is allowed in the spec if the user agent does not wish to implement
    /// parse error recovery (https://html.spec.whatwg.org/#parse-errors)
    pub fn try_parse(mut self) -> HtmlParseResult<Document> {
        if self.encoding_confidence != EncodingConfidence::Certain {
            let (encoding, confidence) = HtmlParser::determine_encoding(&self.input_byte_stream);

            self.character_encoding = encoding;
            self.encoding_confidence = confidence;
        }

        println!(
            "Document Character Encoding: {}",
            self.character_encoding.to_string()
        );
        println!(
            "Document Encoding Confidence: {:?}",
            self.encoding_confidence
        );

        while let Some(c) = self.decode_char()? {
            print!("{c}")
        }

        todo!("parse document")
    }

    /// Will parse an HTML document and recover from any errors as defined in the HTML parsing specification.
    /// (https://html.spec.whatwg.org/#parse-errors)
    #[allow(unused)]
    pub fn parse(self) -> Document {
        todo!("Parse with error recovery")
    }

    /// Decodes bytes from the input_byte_stream in a "lossy" manner (i.e. invalid data is
    /// replaced with REPLACEMENT_CHARACTER)
    #[allow(unused)]
    fn decode_char(&mut self) -> HtmlParseResult<Option<char>> {
        // Use the decoder for the selected character encoding to get a character
        let decoded = self
            .character_encoding
            .decoder()
            .decode(&mut self.input_byte_stream);

        let decoded = match decoded {
            // Replace invalid or incomplete sequences with a replacement character
            Err(DecodingError::InvalidData | DecodingError::UnexpectedEof) => {
                return Ok(Some(char::REPLACEMENT_CHARACTER))
            }

            // Valid encoded data, but invalid character for tokenization
            Err(DecodingError::UnexpectedSurrogate) => {
                return Err(HtmlParseError::SurrogateInInputStream)
            }
            Err(DecodingError::UnexpectedNonCharacter) => {
                return Err(HtmlParseError::NoncharacterInInputStream)
            }
            Err(DecodingError::UnexpectedControl) => {
                return Err(HtmlParseError::ControlCharacterInInputStream)
            }

            // Forward valid input characters from the decoder
            Ok(x) => x,
        };

        // if we got a valid character, extract the code-point and the underlying bytes
        let Some((character, mut bytes)) = decoded else {
            return Ok(None)
        };

        // Append the bytes we read to the running byte tracker
        self.read_bytes.append(&mut bytes);

        Ok(Some(character))
    }

    /// https://html.spec.whatwg.org/#changing-the-encoding-while-parsing
    ///
    /// This algorithm is only invoked when a new encoding is found declared
    /// on a meta element.
    #[allow(unused)]
    fn change_encoding(&mut self, new_encoding: CharacterEncoding) {
        if matches!(
            self.character_encoding,
            CharacterEncoding::Utf16BE | CharacterEncoding::Utf16LE
        ) {
            self.encoding_confidence = EncodingConfidence::Certain;
            return;
        }

        if matches!(
            new_encoding,
            CharacterEncoding::Utf16BE | CharacterEncoding::Utf16LE
        ) {
            self.character_encoding = CharacterEncoding::Utf8;
        }

        if new_encoding == CharacterEncoding::XUserDefined {
            self.character_encoding = CharacterEncoding::Windows1252;
        }

        if new_encoding == self.character_encoding {
            self.encoding_confidence = EncodingConfidence::Certain;
            return;
        }

        if self.is_encoding_equal(new_encoding) {
            self.character_encoding = new_encoding;
            self.encoding_confidence = EncodingConfidence::Certain;
            return;
        }

        // TODO: restart the navigate algorithm
        todo!("restart navigation")
    }

    #[allow(unused)]
    fn is_encoding_equal(&self, new_encoding: CharacterEncoding) -> bool {
        // TODO: Check if all the bytes up to the last byte converted by the
        //       current decoder have the same Unicode interpretations in both
        //       the current encoding and the new encoding

        todo!("check byte equality")
    }

    /// Function that implements the "encoding sniffing algorithm"
    /// defined in the spec (https://html.spec.whatwg.org/#determining-the-character-encoding)
    fn determine_encoding(io_queue: &IoQueue<R>) -> (CharacterEncoding, EncodingConfidence) {
        // Step 1: BOM sniffing
        let bytes = (
            io_queue.peek_nth(0),
            io_queue.peek_nth(1),
            io_queue.peek_nth(2),
        );

        match bytes {
            (Some(0xEF), Some(0xBB), Some(0xBF)) => {
                return (CharacterEncoding::Utf8, EncodingConfidence::Certain)
            }
            (Some(0xFE), Some(0xFF), _) => {
                return (CharacterEncoding::Utf16BE, EncodingConfidence::Certain)
            }
            (Some(0xFF), Some(0xFE), _) => {
                return (CharacterEncoding::Utf16LE, EncodingConfidence::Certain)
            }
            _ => {}
        }

        // Step 2: Explicitly defined user preferences
        // TODO: implement user encoding preference

        // Step 3: Optionally wait for first 1024 bytes to pre-scan?
        io_queue.peek_max(1024);

        // Step 4: Transport layer defined character encoding
        // TODO

        // Step 5: Pre-scan the byte stream to determine the encoding
        if let Some(encoding) = HtmlPreScanner::new(io_queue).pre_scan_byte_stream() {
            return (encoding, EncodingConfidence::Tentative);
        }

        // Step 6: If this HtmlParser is parsing on behalf of a document with a parent,
        //         use the content encoding of the parent document (some caveats) with
        //         confidence tentative
        // TODO

        // Step 7: If UA has information on the likely encoding (from previous visits),
        //         use that with confidence tentative
        // TODO

        // Step 8: Apply frequency analysis to the input stream to autodetect a possible
        //         encoding with confidence tentative. Mostly useful for reading local
        //         files where the entire content can be examined.
        // TODO

        // Step 9: Use implementation defined default encoding
        const DEFAULT_ENCODING: CharacterEncoding = CharacterEncoding::Utf8;

        (DEFAULT_ENCODING, EncodingConfidence::Tentative)
    }
}

#[allow(unused)]
pub struct Document {
    encoding: CharacterEncoding,
}
