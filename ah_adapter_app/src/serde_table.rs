use std::{
    error, fmt,
    io::{self, BufWriter, Write},
};

use serde::{ser, Serialize};

#[derive(Debug)]
pub enum Error {
    CommonError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CommonError(message) => {
                write!(f, "Error when serializing content: {}", message)
            }
        }
    }
}

impl error::Error for Error {}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self::CommonError(msg.to_string())
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::CommonError(format!("{}", err))
    }
}

pub fn print_object<T>(name: &str, object: &T, column_width: usize) -> Result<(), Error>
where
    T: Serialize,
{
    let mut writer = BufWriter::new(io::stdout());
    writeln!(&mut writer, "{}:", name)?;
    to_writer(&mut writer, object, column_width)?;
    write!(&mut writer, "\n\n")?;
    Ok(())
}

pub fn to_writer<W, T>(writer: W, object: &T, column_width: usize) -> Result<(), Error>
where
    T: Serialize,
    W: io::Write,
{
    let mut serializer = Serializer::new(writer, column_width);
    object.serialize(&mut serializer)
}

struct Serializer<W> {
    writer: W,
    column_width: usize,
    indentation: usize,
    first_in_seq: bool,
}

impl<W> Serializer<W>
where
    W: io::Write,
{
    pub fn new(writer: W, column_width: usize) -> Self {
        Self {
            writer,
            column_width,
            indentation: 0,
            first_in_seq: false,
        }
    }

    pub fn serialize_item<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: Serialize,
    {
        if !self.first_in_seq {
            write!(self.writer, "\n")?;
            for _ in 0..self.indentation {
                write!(self.writer, "{:<width$}", "", width = self.column_width)?;
            }
        } else {
            self.first_in_seq = false;
        }
        value.serialize(&mut *self)
    }

    pub fn serialize_indented<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: Serialize,
    {
        self.indentation += 1;
        value.serialize(&mut *self)?;
        self.indentation -= 1;
        Ok(())
    }
}

impl<'a, W> ser::Serializer for &'a mut Serializer<W>
where
    W: io::Write,
{
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
        let s = if v { "true" } else { "false" };
        self.serialize_str(s)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "{:<width$}", v, width = self.column_width)?;
        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_str("<empty>")
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.first_in_seq = true;
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(None)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(None)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.serialize_seq(None)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.serialize_seq(None)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_seq(None)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.serialize_seq(None)
    }
}

impl<'a, W> ser::SerializeSeq for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.serialize_item(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeTuple for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.serialize_item(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeTupleStruct for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.serialize_item(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeTupleVariant for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.serialize_item(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeMap for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();

    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.serialize_item(key)
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.serialize_indented(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeStruct for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.serialize_item(key)?;
        self.serialize_indented(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeStructVariant for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.serialize_item(key)?;
        self.serialize_indented(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
