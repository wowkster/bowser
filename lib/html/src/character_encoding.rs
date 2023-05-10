use core::num;
use std::{io::Read, str::FromStr};

use crate::{io_queue::IoQueue, HtmlParseResult};

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum CharacterEncoding {
    #[default]
    Utf8,
    IBM866,
    ISO8859_2,
    ISO8859_3,
    ISO8859_4,
    ISO8859_5,
    ISO8859_6,
    ISO8859_7,
    ISO8859_8,
    ISO8859_8I,
    ISO8859_10,
    ISO8859_13,
    ISO8859_14,
    ISO8859_15,
    ISO8859_16,
    KOI8R,
    KOI8U,
    Macintosh,
    Windows874,
    Windows1250,
    Windows1251,
    Windows1252,
    Windows1253,
    Windows1254,
    Windows1255,
    Windows1256,
    Windows1257,
    Windows1258,
    XMacCyrillic,
    GBK,
    GB18030,
    Big5,
    EucJp,
    ISO2022Jp,
    ShiftJIS,
    EucKr,
    Replacement,
    Utf16BE,
    Utf16LE,
    XUserDefined,
}

impl FromStr for CharacterEncoding {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use CharacterEncoding::*;

        Ok(match s {
            "unicode-1-1-utf-8" | "unicode11utf8" | "unicode20utf8" | "utf-8" | "utf8"
            | "x-unicode20utf8" => Utf8,
            "866" | "cp866" | "csibm866" | "ibm866" => IBM866,
            "csisolatin2" | "iso-8859-2" | "iso-ir-101" | "iso8859-2" | "iso88592"
            | "iso_8859-2" | "iso_8859-2:1987" | "l2" | "latin2" => ISO8859_2,
            "csisolatin3" | "iso-8859-3" | "iso-ir-109" | "iso8859-3" | "iso88593"
            | "iso_8859-3" | "iso_8859-3:1988" | "l3" | "latin3" => ISO8859_3,
            "csisolatin4" | "iso-8859-4" | "iso-ir-110" | "iso8859-4" | "iso88594"
            | "iso_8859-4" | "iso_8859-4:1988" | "l4" | "latin4" => ISO8859_4,
            "csisolatincyrillic" | "cyrillic" | "iso-8859-5" | "iso-ir-144" | "iso8859-5"
            | "iso88595" | "iso_8859-5" | "iso_8859-5:1988" => ISO8859_5,
            "arabic" | "asmo-708" | "csiso88596e" | "csiso88596i" | "csisolatinarabic"
            | "ecma-114" | "iso-8859-6" | "iso-8859-6-e" | "iso-8859-6-i" | "iso-ir-127"
            | "iso8859-6" | "iso88596" | "iso_8859-6" | "iso_8859-6:1987" => ISO8859_6,
            "csisolatingreek" | "ecma-118" | "elot_928" | "greek" | "greek8" | "iso-8859-7"
            | "iso-ir-126" | "iso8859-7" | "iso88597" | "iso_8859-7" | "iso_8859-7:1987"
            | "sun_eu_greek" => ISO8859_7,
            "csiso88598e" | "csisolatinhebrew" | "hebrew" | "iso-8859-8" | "iso-8859-8-e"
            | "iso-ir-138" | "iso8859-8" | "iso88598" | "iso_8859-8" | "iso_8859-8:1988"
            | "visual" => ISO8859_8,
            "csiso88598i" | "iso-8859-8-i" | "logical" => ISO8859_8I,
            "csisolatin6" | "iso-8859-10" | "iso-ir-157" | "iso8859-10" | "iso885910" | "l6"
            | "latin6" => ISO8859_10,
            "iso-8859-13" | "iso8859-13" | "iso885913" => ISO8859_13,
            "iso-8859-14" | "iso8859-14" | "iso885914" => ISO8859_14,
            "csisolatin9" | "iso-8859-15" | "iso8859-15" | "iso885915" | "iso_8859-15" | "l9" => {
                ISO8859_15
            }
            "iso-8859-16" => ISO8859_16,
            "cskoi8r" | "koi" | "koi8" | "koi8-r" | "koi8_r" => KOI8R,
            "koi8-ru" | "koi8-u" => KOI8U,
            "csmacintosh" | "mac" | "macintosh" | "x-mac-roman" => Macintosh,
            "dos-874" | "iso-8859-11" | "iso8859-11" | "iso885911" | "tis-620" | "windows-874" => {
                Windows874
            }
            "cp1250" | "windows-1250" | "x-cp1250" => Windows1250,
            "cp1251" | "windows-1251" | "x-cp1251" => Windows1251,
            "ansi_x3.4-1968" | "ascii" | "cp1252" | "cp819" | "csisolatin1" | "ibm819"
            | "iso-8859-1" | "iso-ir-100" | "iso8859-1" | "iso88591" | "iso_8859-1"
            | "iso_8859-1:1987" | "l1" | "latin1" | "us-ascii" | "windows-1252" | "x-cp1252" => {
                Windows1252
            }
            "cp1253" | "windows-1253" | "x-cp1253" => Windows1253,
            "cp1254" | "csisolatin5" | "iso-8859-9" | "iso-ir-148" | "iso8859-9" | "iso88599"
            | "iso_8859-9" | "iso_8859-9:1989" | "l5" | "latin5" | "windows-1254" | "x-cp1254" => {
                Windows1254
            }
            "cp1255" | "windows-1255" | "x-cp1255" => Windows1255,
            "cp1256" | "windows-1256" | "x-cp1256" => Windows1256,
            "cp1257" | "windows-1257" | "x-cp1257" => Windows1257,
            "cp1258" | "windows-1258" | "x-cp1258" => Windows1258,
            "x-mac-cyrillic" | "x-mac-ukrainian" => XMacCyrillic,
            "chinese" | "csgb2312" | "csiso58gb231280" | "gb2312" | "gb_2312" | "gb_2312-80"
            | "gbk" | "iso-ir-58" | "x-gbk" => GBK,
            "gb18030" => GB18030,
            "big5" | "big5-hkscs" | "cn-big5" | "csbig5" | "x-x-big5" => Big5,
            "cseucpkdfmtjapanese" | "euc-jp" | "x-euc-jp" => EucJp,
            "csiso2022jp" | "iso-2022-jp" => ISO2022Jp,
            "csshiftjis" | "ms932" | "ms_kanji" | "shift-jis" | "shift_jis" | "sjis"
            | "windows-31j" | "x-sjis" => ShiftJIS,
            "cseuckr" | "csksc56011987" | "euc-kr" | "iso-ir-149" | "korean" | "ks_c_5601-1987"
            | "ks_c_5601-1989" | "ksc5601" | "ksc_5601" | "windows-949" => EucKr,
            "csiso2022kr" | "hz-gb-2312" | "iso-2022-cn" | "iso-2022-cn-ext" | "iso-2022-kr"
            | "replacement" => Replacement,
            "unicodefffe" | "utf-16be" => Utf16BE,
            "csunicode" | "iso-10646-ucs-2" | "ucs-2" | "unicode" | "unicodefeff" | "utf-16"
            | "utf-16le" => Utf16LE,
            "x-user-defined" => XUserDefined,
            _ => return Err(()),
        })
    }
}

impl ToString for CharacterEncoding {
    fn to_string(&self) -> String {
        match self {
            CharacterEncoding::Utf8 => "UTF-8",
            CharacterEncoding::IBM866 => "IBM866",
            CharacterEncoding::ISO8859_2 => "ISO-8859-2",
            CharacterEncoding::ISO8859_3 => "ISO-8859-3",
            CharacterEncoding::ISO8859_4 => "ISO-8859-4",
            CharacterEncoding::ISO8859_5 => "ISO-8859-5",
            CharacterEncoding::ISO8859_6 => "ISO-8859-6",
            CharacterEncoding::ISO8859_7 => "ISO-8859-7",
            CharacterEncoding::ISO8859_8 => "ISO-8859-8",
            CharacterEncoding::ISO8859_8I => "ISO-8859-8-I",
            CharacterEncoding::ISO8859_10 => "ISO-8859-10",
            CharacterEncoding::ISO8859_13 => "ISO-8859-13",
            CharacterEncoding::ISO8859_14 => "ISO-8859-14",
            CharacterEncoding::ISO8859_15 => "ISO-8859-15",
            CharacterEncoding::ISO8859_16 => "ISO-8859-16",
            CharacterEncoding::KOI8R => "KOI8-R",
            CharacterEncoding::KOI8U => "KOI8-U",
            CharacterEncoding::Macintosh => "macintosh",
            CharacterEncoding::Windows874 => "windows-874",
            CharacterEncoding::Windows1250 => "windows-1250",
            CharacterEncoding::Windows1251 => "windows-1251",
            CharacterEncoding::Windows1252 => "windows-1252",
            CharacterEncoding::Windows1253 => "windows-1253",
            CharacterEncoding::Windows1254 => "windows-1254",
            CharacterEncoding::Windows1255 => "windows-1255",
            CharacterEncoding::Windows1256 => "windows-1256",
            CharacterEncoding::Windows1257 => "windows-1257",
            CharacterEncoding::Windows1258 => "windows-1258",
            CharacterEncoding::XMacCyrillic => "x-mac-cyrillic",
            CharacterEncoding::GBK => "GBK",
            CharacterEncoding::GB18030 => "gb18030",
            CharacterEncoding::Big5 => "Big5",
            CharacterEncoding::EucJp => "EUC-JP",
            CharacterEncoding::ISO2022Jp => "ISO-2022-JP",
            CharacterEncoding::ShiftJIS => "Shift_JIS",
            CharacterEncoding::EucKr => "EUC-KR",
            CharacterEncoding::Replacement => "replacement",
            CharacterEncoding::Utf16BE => "UTF-16BE",
            CharacterEncoding::Utf16LE => "UTF-16LE",
            CharacterEncoding::XUserDefined => "x-user-defined",
        }
        .to_owned()
    }
}

impl CharacterEncoding {
    pub fn decoder<R: Read>(&self) -> impl IoDecoder<R> {
        match self {
            CharacterEncoding::Utf8 => Utf8Decoder,
            CharacterEncoding::IBM866 => todo!(),
            CharacterEncoding::ISO8859_2 => todo!(),
            CharacterEncoding::ISO8859_3 => todo!(),
            CharacterEncoding::ISO8859_4 => todo!(),
            CharacterEncoding::ISO8859_5 => todo!(),
            CharacterEncoding::ISO8859_6 => todo!(),
            CharacterEncoding::ISO8859_7 => todo!(),
            CharacterEncoding::ISO8859_8 => todo!(),
            CharacterEncoding::ISO8859_8I => todo!(),
            CharacterEncoding::ISO8859_10 => todo!(),
            CharacterEncoding::ISO8859_13 => todo!(),
            CharacterEncoding::ISO8859_14 => todo!(),
            CharacterEncoding::ISO8859_15 => todo!(),
            CharacterEncoding::ISO8859_16 => todo!(),
            CharacterEncoding::KOI8R => todo!(),
            CharacterEncoding::KOI8U => todo!(),
            CharacterEncoding::Macintosh => todo!(),
            CharacterEncoding::Windows874 => todo!(),
            CharacterEncoding::Windows1250 => todo!(),
            CharacterEncoding::Windows1251 => todo!(),
            CharacterEncoding::Windows1252 => todo!(),
            CharacterEncoding::Windows1253 => todo!(),
            CharacterEncoding::Windows1254 => todo!(),
            CharacterEncoding::Windows1255 => todo!(),
            CharacterEncoding::Windows1256 => todo!(),
            CharacterEncoding::Windows1257 => todo!(),
            CharacterEncoding::Windows1258 => todo!(),
            CharacterEncoding::XMacCyrillic => todo!(),
            CharacterEncoding::GBK => todo!(),
            CharacterEncoding::GB18030 => todo!(),
            CharacterEncoding::Big5 => todo!(),
            CharacterEncoding::EucJp => todo!(),
            CharacterEncoding::ISO2022Jp => todo!(),
            CharacterEncoding::ShiftJIS => todo!(),
            CharacterEncoding::EucKr => todo!(),
            CharacterEncoding::Replacement => todo!(),
            CharacterEncoding::Utf16BE => todo!(),
            CharacterEncoding::Utf16LE => todo!(),
            CharacterEncoding::XUserDefined => todo!(),
        }
    }
}

pub trait IoDecoder<R: Read> {
    fn decode(&self, io_queue: &mut IoQueue<R>) -> Option<(char, Vec<u8>)>;
}

pub struct Utf8Decoder;

impl<R: Read> IoDecoder<R> for Utf8Decoder {
    fn decode(&self, io_queue: &mut IoQueue<R>) -> Option<(char, Vec<u8>)> {
        let start_position = io_queue.bytes_read();

        let first_bytes = io_queue.peek_arr(4);

        let c = utf8_decode::decode(io_queue);

        let end_position = io_queue.bytes_read();

        let num_bytes = end_position - start_position;

        let read_bytes = (&first_bytes[..num_bytes]).to_vec();

        let Some(c) = c else {
            return None
        };

        let Ok(c) = c else {
            return None
        };

        Some((c, read_bytes))
    }
}
