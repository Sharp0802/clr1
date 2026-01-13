use serde::{ser, Serialize};
use std::fmt::{Debug, Display};
use std::io::Write;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Encoding error: {0}")]
    Encoding(#[from] FromUtf8Error),
    #[error("Serde error: {0}")]
    Custom(String),
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

struct Ser<W: Write> {
    writer: W,
    in_range: bool,
    indent: usize,
    stack: Vec<bool>,
}

impl<W: Write> Ser<W> {
    fn new(writer: W, indent: usize) -> Ser<W> {
        Ser {
            writer,
            in_range: false,
            indent,
            stack: vec![false],
        }
    }

    fn serialize<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        value.serialize(self)
    }

    fn begin_item(&mut self, indent: bool) -> Result<(), Error> {
        if self.peek() {
            write!(self.writer, ",")?;
        }

        if indent {
            write!(self.writer, "\n")?;
            self.do_indent()?;
        } else {
            write!(self.writer, " ")?;
        }

        *self.peek_mut() = true;
        Ok(())
    }

    fn serialize_item<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
        indent: bool,
    ) -> Result<(), Error> {
        self.begin_item(indent)?;
        self.serialize(value)?;
        Ok(())
    }

    fn peek(&self) -> bool {
        *self.stack.last().unwrap()
    }

    fn peek_mut(&mut self) -> &mut bool {
        self.stack.last_mut().unwrap()
    }

    fn do_indent(&mut self) -> Result<(), Error> {
        write!(
            self.writer,
            "{}",
            "    ".repeat(self.indent + self.stack.len() - 1)
        )?;
        Ok(())
    }

    fn indent(&mut self) -> &mut Self {
        self.stack.push(false);
        self
    }

    fn outdent(&mut self) -> Result<(), Error> {
        if self.stack.pop() == Some(true) {
            write!(self.writer, "\n")?;
            self.do_indent()?;
        }

        if self.stack.is_empty() {
            self.stack.push(false);
        }

        Ok(())
    }
}

impl<W: Write> ser::Serializer for &mut Ser<W> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "{}", v)?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "{}", v)?;
        Ok(())
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "{}", v)?;
        Ok(())
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "{}", v)?;
        Ok(())
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "{}", v)?;
        Ok(())
    }
    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "{}", v)?;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "{}", v)?;
        Ok(())
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "{}", v)?;
        Ok(())
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "{}", v)?;
        Ok(())
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "{}", v)?;
        Ok(())
    }
    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "{}", v)?;
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "{}f32", v)?;
        Ok(())
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "{}f64", v)?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "'{}'", v)?;
        Ok(())
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "\"{}\"", v)?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "&[")?;
        for b in v {
            write!(self.writer, "0x{:02x}u8", b)?;
        }
        write!(self.writer, "]")?;
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "None")?;
        Ok(())
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "Some(")?;
        value.serialize(&mut *self)?;
        write!(self.writer, ")")?;
        Ok(())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "()")?;
        Ok(())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "{}", name)?;
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "{}::{}", name, variant)?;
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        if name == "Boxed" {
            write!(self.writer, "&")?;
            self.serialize(value)?;
        } else {
            write!(self.writer, "{}(", name)?;
            self.serialize(value)?;
            write!(self.writer, ")")?;
        }

        Ok(())
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        _: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "{}::{}(", name, variant)?;
        self.serialize(value)?;
        write!(self.writer, ")")?;
        Ok(())
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        write!(self.writer, "&[")?;
        Ok(self.indent())
    }

    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
        write!(self.writer, "(")?;
        Ok(self.indent())
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        write!(self.writer, "{}(", name)?;
        Ok(self.indent())
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        write!(self.writer, "{}::{}(", name, variant)?;
        Ok(self.indent())
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        write!(self.writer, "vec![")?;
        Ok(self.indent())
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        if name == "RangeInclusive" {
            self.in_range = true;
        } else {
            write!(self.writer, "{} {{", name)?;
        }
        Ok(self.indent())
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        write!(self.writer, "{}::{} {{", name, variant)?;
        Ok(self.indent())
    }
}

impl<W: Write> ser::SerializeSeq for &mut Ser<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.serialize_item(value, true)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.outdent()?;
        write!(self.writer, "]")?;
        Ok(())
    }
}

impl<W: Write> ser::SerializeTuple for &mut Ser<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.serialize_item(value, false)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.outdent()?;
        write!(self.writer, ")")?;
        Ok(())
    }
}

impl<W: Write> ser::SerializeTupleStruct for &mut Ser<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.serialize_item(value, false)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeTuple::end(self)
    }
}

impl<W: Write> ser::SerializeTupleVariant for &mut Ser<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.serialize_item(value, false)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeTuple::end(self)
    }
}

impl<W: Write> ser::SerializeMap for &mut Ser<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        self.begin_item(true)?;
        write!(self.writer, "(")?;
        self.serialize(key)?;
        write!(self.writer, ", ")?;
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<(), Self::Error> {
        self.serialize(_value)?;
        write!(self.writer, ")")?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.outdent()?;
        write!(self.writer, "].into()")?;
        Ok(())
    }
}

impl<W: Write> ser::SerializeStruct for &mut Ser<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        if self.in_range {
            self.serialize(value)?;
            if key == "start" {
                write!(self.writer, "..=")?;
            }
        } else {
            self.begin_item(true)?;
            write!(self.writer, "{}: ", key)?;
            self.serialize(value)?;
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.outdent()?;
        if self.in_range {
            self.in_range = false;
        } else {
            write!(self.writer, "}}")?;
        }
        Ok(())
    }
}

impl<W: Write> ser::SerializeStructVariant for &mut Ser<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        ser::SerializeStruct::serialize_field(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeStruct::end(self)
    }
}

#[derive(Default)]
pub struct Options {
    pub initial_indent: usize,
}

impl Options {
    fn to_ser<W: Write>(&self, writer: W) -> Ser<W> {
        Ser::new(writer, self.initial_indent)
    }
}

pub fn serialize<T: ?Sized + Serialize, W: Write>(
    writer: &mut W,
    value: &T,
    options: Options,
) -> Result<(), Error> {
    options.to_ser(writer).serialize(value)
}

pub fn to_string<T: ?Sized + Serialize>(value: &T, options: Options) -> Result<String, Error> {
    let mut buffer = Vec::new();
    serialize(&mut buffer, value, options)?;
    Ok(String::from_utf8(buffer)?)
}
