//! Hjson Serialization
//!
//! This module provides for Hjson serialization with the type `Serializer`.

use std::io;
use std::num::FpCategory;
use std::fmt::{ Display, LowerExp };

use serde::ser;
use super::error::{Error, ErrorCode, Result};

use super::util::ParseNumber;

use regex::Regex;

/// A structure for serializing Rust values into Hjson.
pub struct Serializer<W, F> {
    writer: W,
    formatter: F,

    /// `first` is used to signify if we should print a comma when we are walking through a
    /// sequence.
    first: bool,
}

impl<'a, W> Serializer<W, HjsonFormatter<'a>>
    where W: io::Write {
    /// Creates a new Hjson serializer.
    #[inline]
    pub fn new(writer: W) -> Self {
        Serializer::with_formatter(writer, HjsonFormatter::new())
    }
}

impl<W, F> Serializer<W, F>
    where W: io::Write,
          F: Formatter {
    /// Creates a new Hjson visitor whose output will be written to the writer
    /// specified.
    #[inline]
    pub fn with_formatter(writer: W, formatter: F) -> Self {
        Serializer {
            writer: writer,
            formatter: formatter,
            first: false,
        }
    }

    /// Unwrap the `Writer` from the `Serializer`.
    #[inline]
    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl<W, F> ser::Serializer for Serializer<W, F>
    where W: io::Write,
          F: Formatter {
    type Error = Error;

    #[inline]
    fn serialize_bool(&mut self, value: bool) -> Result<()> {
        try!(self.formatter.start_value(&mut self.writer));
        if value {
            self.writer.write_all(b"true").map_err(From::from)
        } else {
            self.writer.write_all(b"false").map_err(From::from)
        }
    }

    #[inline]
    fn serialize_isize(&mut self, value: isize) -> Result<()> {
        try!(self.formatter.start_value(&mut self.writer));
        write!(&mut self.writer, "{}", value).map_err(From::from)
    }

    #[inline]
    fn serialize_i8(&mut self, value: i8) -> Result<()> {
        try!(self.formatter.start_value(&mut self.writer));
        write!(&mut self.writer, "{}", value).map_err(From::from)
    }

    #[inline]
    fn serialize_i16(&mut self, value: i16) -> Result<()> {
        try!(self.formatter.start_value(&mut self.writer));
        write!(&mut self.writer, "{}", value).map_err(From::from)
    }

    #[inline]
    fn serialize_i32(&mut self, value: i32) -> Result<()> {
        try!(self.formatter.start_value(&mut self.writer));
        write!(&mut self.writer, "{}", value).map_err(From::from)
    }

    #[inline]
    fn serialize_i64(&mut self, value: i64) -> Result<()> {
        try!(self.formatter.start_value(&mut self.writer));
        write!(&mut self.writer, "{}", value).map_err(From::from)
    }

    #[inline]
    fn serialize_usize(&mut self, value: usize) -> Result<()> {
        try!(self.formatter.start_value(&mut self.writer));
        write!(&mut self.writer, "{}", value).map_err(From::from)
    }

    #[inline]
    fn serialize_u8(&mut self, value: u8) -> Result<()> {
        try!(self.formatter.start_value(&mut self.writer));
        write!(&mut self.writer, "{}", value).map_err(From::from)
    }

    #[inline]
    fn serialize_u16(&mut self, value: u16) -> Result<()> {
        try!(self.formatter.start_value(&mut self.writer));
        write!(&mut self.writer, "{}", value).map_err(From::from)
    }

    #[inline]
    fn serialize_u32(&mut self, value: u32) -> Result<()> {
        try!(self.formatter.start_value(&mut self.writer));
        write!(&mut self.writer, "{}", value).map_err(From::from)
    }

    #[inline]
    fn serialize_u64(&mut self, value: u64) -> Result<()> {
        try!(self.formatter.start_value(&mut self.writer));
        write!(&mut self.writer, "{}", value).map_err(From::from)
    }

    #[inline]
    fn serialize_f32(&mut self, value: f32) -> Result<()> {
        try!(self.formatter.start_value(&mut self.writer));
        fmt_f32_or_null(&mut self.writer, if value == -0f32 { 0f32 } else { value }).map_err(From::from)
    }

    #[inline]
    fn serialize_f64(&mut self, value: f64) -> Result<()> {
        try!(self.formatter.start_value(&mut self.writer));
        fmt_f64_or_null(&mut self.writer, if value == -0f64 { 0f64 } else { value }).map_err(From::from)
    }

    #[inline]
    fn serialize_char(&mut self, value: char) -> Result<()> {
        try!(self.formatter.start_value(&mut self.writer));
        escape_char(&mut self.writer, value).map_err(From::from)
    }

    #[inline]
    fn serialize_str(&mut self, value: &str) -> Result<()> {
        quote_str(&mut self.writer, &mut self.formatter, value).map_err(From::from)
    }

    #[inline]
    fn serialize_none(&mut self) -> Result<()> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<V>(&mut self, value: V) -> Result<()>
        where V: ser::Serialize {
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit(&mut self) -> Result<()> {
        try!(self.formatter.start_value(&mut self.writer));
        self.writer.write_all(b"null").map_err(From::from)
    }

    /// Override `visit_newtype_struct` to serialize newtypes without an object wrapper.
    #[inline]
    fn serialize_newtype_struct<T>(&mut self,
                               _name: &'static str,
                               value: T) -> Result<()>
        where T: ser::Serialize {
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit_variant(&mut self,
                          _name: &str,
                          _variant_index: usize,
                          variant: &str) -> Result<()> {
        try!(self.formatter.open(&mut self.writer, b'{'));
        try!(self.formatter.comma(&mut self.writer, true));
        try!(escape_key(&mut self.writer, variant));
        try!(self.formatter.colon(&mut self.writer));
        try!(self.writer.write_all(b"[]"));
        self.formatter.close(&mut self.writer, b'}')
    }

    #[inline]
    fn serialize_newtype_variant<T>(&mut self,
                                _name: &str,
                                _variant_index: usize,
                                variant: &str,
                                value: T) -> Result<()>
        where T: ser::Serialize {
        try!(self.formatter.open(&mut self.writer, b'{'));
        try!(self.formatter.comma(&mut self.writer, true));
        try!(escape_key(&mut self.writer, variant));
        try!(self.formatter.colon(&mut self.writer));
        try!(value.serialize(self));
        self.formatter.close(&mut self.writer, b'}')
    }

    #[inline]
    fn serialize_seq<V>(&mut self, mut visitor: V) -> Result<()>
        where V: ser::SeqVisitor {
        match visitor.len() {
            Some(len) if len == 0 => {
                try!(self.formatter.start_value(&mut self.writer));
                self.writer.write_all(b"[]").map_err(From::from)
            }
            _ => {
                try!(self.formatter.open(&mut self.writer, b'['));

                self.first = true;

                while let Some(()) = try!(visitor.visit(self)) { }

                self.formatter.close(&mut self.writer, b']').map_err(From::from)
            }
        }

    }

    #[inline]
    fn serialize_tuple_variant<V>(&mut self,
                              _name: &str,
                              _variant_index: usize,
                              variant: &str,
                              visitor: V) -> Result<()>
        where V: ser::SeqVisitor {
        try!(self.formatter.open(&mut self.writer, b'{'));
        try!(self.formatter.comma(&mut self.writer, true));
        try!(escape_key(&mut self.writer, variant));
        try!(self.formatter.colon(&mut self.writer));
        try!(self.serialize_seq(visitor));
        self.formatter.close(&mut self.writer, b'}')
    }

    #[inline]
    fn serialize_seq_elt<T>(&mut self, value: T) -> Result<()>
        where T: ser::Serialize {
        try!(self.formatter.comma(&mut self.writer, self.first));
        try!(value.serialize(self));

        self.first = false;

        Ok(())
    }

    #[inline]
    fn serialize_map<V>(&mut self, mut visitor: V) -> Result<()>
        where V: ser::MapVisitor {
        match visitor.len() {
            Some(len) if len == 0 => {
                try!(self.formatter.start_value(&mut self.writer));
                self.writer.write_all(b"{}").map_err(From::from)
            }
            _ => {
                try!(self.formatter.open(&mut self.writer, b'{'));

                self.first = true;

                while let Some(()) = try!(visitor.visit(self)) { }

                self.formatter.close(&mut self.writer, b'}')
            }
        }
    }

    #[inline]
    fn serialize_struct_variant<V>(&mut self,
                               _name: &str,
                               _variant_index: usize,
                               variant: &str,
                               visitor: V) -> Result<()>
        where V: ser::MapVisitor {
        try!(self.formatter.open(&mut self.writer, b'{'));
        try!(self.formatter.comma(&mut self.writer, true));
        try!(escape_key(&mut self.writer, variant));
        try!(self.formatter.colon(&mut self.writer));
        try!(self.serialize_map(visitor));

        self.formatter.close(&mut self.writer, b'}')
    }

    #[inline]
    fn serialize_map_elt<K, V>(&mut self, key: K, value: V) -> Result<()>
        where K: ser::Serialize,
              V: ser::Serialize {
        try!(self.formatter.comma(&mut self.writer, self.first));

        try!(key.serialize(&mut MapKeySerializer { ser: self }));
        try!(self.formatter.colon(&mut self.writer));
        try!(value.serialize(self));

        self.first = false;

        Ok(())
    }
}

struct MapKeySerializer<'a, W: 'a, F: 'a> {
    ser: &'a mut Serializer<W, F>,
}

impl<'a, W, F> ser::Serializer for MapKeySerializer<'a, W, F>
    where W: io::Write,
          F: Formatter {
    type Error = Error;

    #[inline]
    fn serialize_str(&mut self, value: &str) -> Result<()> {
        escape_key(&mut self.ser.writer, value).map_err(From::from)
    }

    fn serialize_bool(&mut self, _value: bool) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_i64(&mut self, _value: i64) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_u64(&mut self, _value: u64) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_f64(&mut self, _value: f64) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_unit(&mut self) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_none(&mut self) -> Result<()> {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_some<V>(&mut self, _value: V) -> Result<()>
        where V: ser::Serialize {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_seq<V>(&mut self, _visitor: V) -> Result<()>
        where V: ser::SeqVisitor {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_seq_elt<T>(&mut self, _value: T) -> Result<()>
        where T: ser::Serialize {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_map<V>(&mut self, _visitor: V) -> Result<()>
        where V: ser::MapVisitor {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }

    fn serialize_map_elt<K, V>(&mut self, _key: K, _value: V) -> Result<()>
        where K: ser::Serialize,
              V: ser::Serialize {
        Err(Error::Syntax(ErrorCode::KeyMustBeAString, 0, 0))
    }
}

/// This trait abstracts away serializing the JSON control characters
pub trait Formatter {
    /// Called when serializing a '{' or '['.
    fn open<W>(&mut self, writer: &mut W, ch: u8) -> Result<()>
        where W: io::Write;

    /// Called when serializing a ','.
    fn comma<W>(&mut self, writer: &mut W, first: bool) -> Result<()>
        where W: io::Write;

    /// Called when serializing a ':'.
    fn colon<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write;

    /// Called when serializing a '}' or ']'.
    fn close<W>(&mut self, writer: &mut W, ch: u8) -> Result<()>
        where W: io::Write;

    /// Newline with indent.
    fn newline<W>(&mut self, writer: &mut W, add_indent: i32) -> Result<()>
        where W: io::Write;

    /// Start a value.
    fn start_value<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write;
}

struct HjsonFormatter<'a> {
    current_indent: usize,
    current_is_array: bool,
    stack: Vec<bool>,
    at_colon: bool,
    indent: &'a [u8],
    braces_same_line: bool,
}

impl<'a> HjsonFormatter<'a> {
    /// Construct a formatter that defaults to using two spaces for indentation.
    pub fn new() -> Self {
        HjsonFormatter::with_indent(b"  ")
    }

    /// Construct a formatter that uses the `indent` string for indentation.
    pub fn with_indent(indent: &'a [u8]) -> Self {
        HjsonFormatter {
            current_indent: 0,
            current_is_array: false,
            stack: Vec::new(),
            at_colon: false,
            indent: indent,
            braces_same_line: false,
        }
    }
}

impl<'a> Formatter for HjsonFormatter<'a> {
    fn open<W>(&mut self, writer: &mut W, ch: u8) -> Result<()>
        where W: io::Write {
        if self.current_indent > 0 && !self.current_is_array && !self.braces_same_line {
            try!(self.newline(writer, 0));
        } else {
            try!(self.start_value(writer));
        }
        self.current_indent += 1;
        self.stack.push(self.current_is_array);
        self.current_is_array = ch == b'[';
        writer.write_all(&[ch]).map_err(From::from)
    }

    fn comma<W>(&mut self, writer: &mut W, _: bool) -> Result<()>
        where W: io::Write {
        try!(writer.write_all(b"\n"));
        indent(writer, self.current_indent, self.indent)
    }

    fn colon<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write {
        self.at_colon = !self.braces_same_line;
        writer.write_all(if self.braces_same_line { b": " } else { b":" }).map_err(From::from)
    }

    fn close<W>(&mut self, writer: &mut W, ch: u8) -> Result<()>
        where W: io::Write {
        self.current_indent -= 1;
        self.current_is_array = self.stack.pop().unwrap();
        try!(writer.write(b"\n"));
        try!(indent(writer, self.current_indent, self.indent));
        writer.write_all(&[ch]).map_err(From::from)
    }

    fn newline<W>(&mut self, writer: &mut W, add_indent: i32) -> Result<()>
        where W: io::Write {
        self.at_colon = false;
        try!(writer.write_all(b"\n"));
        let ii = self.current_indent as i32 + add_indent;
        indent(writer, if ii < 0 { 0 } else { ii as usize }, self.indent)
    }

    fn start_value<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write {
        if self.at_colon {
            self.at_colon = false;
            try!(writer.write_all(b" "))
        }
        Ok(())
    }
}

/// Serializes and escapes a `&[u8]` into a Hjson string.
#[inline]
pub fn escape_bytes<W>(wr: &mut W, bytes: &[u8]) -> Result<()>
    where W: io::Write {
    try!(wr.write_all(b"\""));

    let mut start = 0;

    for (i, byte) in bytes.iter().enumerate() {
        let escaped = match *byte {
            b'"' => b"\\\"",
            b'\\' => b"\\\\",
            b'\x08' => b"\\b",
            b'\x0c' => b"\\f",
            b'\n' => b"\\n",
            b'\r' => b"\\r",
            b'\t' => b"\\t",
            _ => { continue; }
        };

        if start < i {
            try!(wr.write_all(&bytes[start..i]));
        }

        try!(wr.write_all(escaped));

        start = i + 1;
    }

    if start != bytes.len() {
        try!(wr.write_all(&bytes[start..]));
    }

    try!(wr.write_all(b"\""));
    Ok(())
}

/// Serializes and escapes a `&str` into a Hjson string.
#[inline]
pub fn quote_str<W, F>(wr: &mut W, formatter: &mut F, value: &str) -> Result<()>
    where W: io::Write,
          F: Formatter {
    lazy_static! {
        // NEEDS_ESCAPE is used to detect characters
        static ref NEEDS_ESCAPE: Regex = Regex::new("[\\\\\"\x00-\x1f\x7f-\u{9f}\u{00ad}\u{0600}-\u{0604}\u{070f}\u{17b4}\u{17b5}\u{200c}-\u{200f}\u{2028}-\u{202f}\u{2060}-\u{206f}\u{feff}\u{fff0}-\u{ffff}]").unwrap();
        // like NEEDS_ESCAPE but without \\ and \"
        static ref NEEDS_QUOTES: Regex = Regex::new("[\x00-\x1f\x7f-\u{9f}\u{00ad}\u{0600}-\u{0604}\u{070f}\u{17b4}\u{17b5}\u{200c}-\u{200f}\u{2028}-\u{202f}\u{2060}-\u{206f}\u{feff}\u{fff0}-\u{ffff}]").unwrap();
        static ref NEEDS_QUOTES2: Regex = Regex::new(r#"^\s|^"|^'''|^#|^/\*|^//|^\{|^\[|\s$"#).unwrap();
        // ''' || (needsQuotes but without \n and \r)
        static ref NEEDS_ESCAPEML: Regex = Regex::new("'''|[\x00-\x09\x0b\x0c\x0e-\x1f\x7f-\u{9f}\u{00ad}\u{0600}-\u{0604}\u{070f}\u{17b4}\u{17b5}\u{200c}-\u{200f}\u{2028}-\u{202f}\u{2060}-\u{206f}\u{feff}\u{fff0}-\u{ffff}]").unwrap();
        // starts with a keyword and optionally is followed by a comment
        static ref STARTS_WITH_KEYWORD: Regex = Regex::new(r#"^(true|false|null)\s*((,|\]|\}|#|//|/\*).*)?$"#).unwrap();
    }

    if value.len() == 0 {
        try!(formatter.start_value(wr));
        return escape_bytes(wr, value.as_bytes());
    }

    // Check if we can insert this string without quotes
    // see hjson syntax (must not parse as true, false, null or number)

    let mut pn = ParseNumber::new(value.bytes());
    let is_number = match pn.parse(true) {
        Ok(_) => true,
        Err(_) => false,
    };

    if is_number || NEEDS_QUOTES.is_match(value) || NEEDS_QUOTES2.is_match(value) || STARTS_WITH_KEYWORD.is_match(value) {

        // First check if the string can be expressed in multiline format or
        // we must replace the offending characters with safe escape sequences.

        if NEEDS_ESCAPE.is_match(value) && !NEEDS_ESCAPEML.is_match(value)  /* && !isRootObject */ {
            ml_str(wr, formatter, value)
        } else {
            try!(formatter.start_value(wr));
            escape_bytes(wr, value.as_bytes())
        }
    }
    else {
        // without quotes
        try!(formatter.start_value(wr));
        wr.write_all(value.as_bytes()).map_err(From::from)
    }
}

/// Serializes and escapes a `&str` into a multiline Hjson string.
pub fn ml_str<W, F>(wr: &mut W, formatter: &mut F, value: &str) -> Result<()>
    where W: io::Write,
          F: Formatter {

    // wrap the string into the ''' (multiline) format

    let a: Vec<&str> = value.split("\n").collect();

    if a.len() == 1 {
        // The string contains only a single line. We still use the multiline
        // format as it avoids escaping the \ character (e.g. when used in a
        // regex).
        try!(formatter.start_value(wr));
        try!(wr.write_all(b"'''"));
        try!(wr.write_all(a[0].as_bytes()));
        try!(wr.write_all(b"'''"))
    } else {
        try!(formatter.newline(wr, 1));
        try!(wr.write_all(b"'''"));
        for line in a {
            try!(formatter.newline(wr, if line.len() > 0 { 1 } else { -999 }));
            try!(wr.write_all(line.as_bytes()));
        }
        try!(formatter.newline(wr, 1));
        try!(wr.write_all(b"'''"));
    }
    Ok(())
}

/// Serializes and escapes a `&str` into a Hjson key.
#[inline]
pub fn escape_key<W>(wr: &mut W, value: &str) -> Result<()>
    where W: io::Write {

    lazy_static! {
        static ref NEEDS_ESCAPE_NAME: Regex = Regex::new(r#"[,\{\[\}\]\s:#"]|//|/\*|'''|^$"#).unwrap();
    }

    // Check if we can insert this name without quotes
    if NEEDS_ESCAPE_NAME.is_match(value) {
        escape_bytes(wr, value.as_bytes()).map_err(From::from)
    } else {
        wr.write_all(value.as_bytes()).map_err(From::from)
    }
}

#[inline]
fn escape_char<W>(wr: &mut W, value: char) -> Result<()>
    where W: io::Write {
    // FIXME: this allocation is required in order to be compatible with stable
    // rust, which doesn't support encoding a `char` into a stack buffer.
    let mut s = String::new();
    s.push(value);
    escape_bytes(wr, s.as_bytes())
}

fn fmt_f32_or_null<W>(wr: &mut W, value: f32) -> Result<()>
    where W: io::Write {
    match value.classify() {
        FpCategory::Nan | FpCategory::Infinite => {
            try!(wr.write_all(b"null"))
        }
        _ => {
            try!(wr.write_all(fmt_small(value).as_bytes()))
        }
    }

    Ok(())
}

fn fmt_f64_or_null<W>(wr: &mut W, value: f64) -> Result<()>
    where W: io::Write {
    match value.classify() {
        FpCategory::Nan | FpCategory::Infinite => {
            try!(wr.write_all(b"null"))
        }
        _ => {
            try!(wr.write_all(fmt_small(value).as_bytes()))
        }
    }

    Ok(())
}

fn indent<W>(wr: &mut W, n: usize, s: &[u8]) -> Result<()>
    where W: io::Write {
    for _ in 0 .. n {
        try!(wr.write_all(s));
    }

    Ok(())
}

// format similar to es6
fn fmt_small<N>(value: N) -> String
    where N: Display + LowerExp {
    let f1 = format!("{}", value);
    let f2 = format!("{:e}", value);
    if f1.len() <= f2.len() + 1 {
        f1
    } else {
        if !f2.contains("e-") { f2.replace("e", "e+") } else { f2 }
    }
}


/// Encode the specified struct into a Hjson `[u8]` writer.
#[inline]
pub fn to_writer<W, T>(writer: &mut W, value: &T) -> Result<()>
    where W: io::Write,
          T: ser::Serialize {
    let mut ser = Serializer::new(writer);
    try!(value.serialize(&mut ser));
    Ok(())
}

/// Encode the specified struct into a Hjson `[u8]` buffer.
#[inline]
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
    where T: ser::Serialize {
    // We are writing to a Vec, which doesn't fail. So we can ignore
    // the error.
    let mut writer = Vec::with_capacity(128);
    try!(to_writer(&mut writer, value));
    Ok(writer)
}

/// Encode the specified struct into a Hjson `String` buffer.
#[inline]
pub fn to_string<T>(value: &T) -> Result<String>
    where T: ser::Serialize {
    let vec = try!(to_vec(value));
    let string = try!(String::from_utf8(vec));
    Ok(string)
}
