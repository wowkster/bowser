use std::io::Read;

use crate::{character_encoding::CharacterEncoding, io_queue::IoQueue, prescan::HtmlPreScanner, HtmlParseResult};

pub struct HtmlParser {
    character_encoding: CharacterEncoding,
    encoding_confidence: EncodingConfidence,
}

/// https://html.spec.whatwg.org/#concept-encoding-confidence
#[derive(Debug, PartialEq, Eq)]
enum EncodingConfidence {
    Tentative,
    Certain,
    Irrelevant,
}

impl HtmlParser {
    pub fn new() -> Self {
        Self {
            character_encoding: CharacterEncoding::default(),
            encoding_confidence: EncodingConfidence::Tentative,
        }
    }

    /// https://html.spec.whatwg.org/#parsing-with-a-known-character-encoding
    pub fn with_definite_encoding(character_encoding: CharacterEncoding) -> Self {
        Self {
            character_encoding,
            encoding_confidence: EncodingConfidence::Certain,
        }
    }

    /// Will try to parse an HTML document, but will abort if any error condition is discovered.
    /// This behavior is allowed in the spec if the user agent does not wish to implement
    /// parse error recovery (https://html.spec.whatwg.org/#parse-errors)
    pub fn try_parse(mut self, input_byte_stream: impl Read) -> HtmlParseResult<Document> {
        let input_byte_stream = IoQueue::new(input_byte_stream);

        if self.encoding_confidence != EncodingConfidence::Certain {
            let (encoding, confidence) = HtmlParser::determine_encoding(&input_byte_stream);

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

        todo!("parse document")
    }

    /// Will parse an HTML document and recover from any errors as defined in the HTML parsing specification.
    /// (https://html.spec.whatwg.org/#parse-errors)
    pub fn parse(self, input_byte_stream: impl Read) -> Document {
        todo!("Parse with error recovery")
    }

    /// Function that implements the "encoding sniffing algorithm"
    /// defined in the spec (https://html.spec.whatwg.org/#determining-the-character-encoding)
    fn determine_encoding<R: Read>(
        io_queue: &IoQueue<R>,
    ) -> (CharacterEncoding, EncodingConfidence) {
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
        if let Some(encoding) = HtmlPreScanner::new(&io_queue).pre_scan_byte_stream() {
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

pub struct Document {
    encoding: CharacterEncoding,
}
