use crate::ser::error::Error;
use dtoa::Floating;
use itoa::Integer;
use serde::{
    ser::{Error as _, Impossible, SerializeStruct},
    Serialize, Serializer,
};
use std::fmt::Display;

#[allow(missing_debug_implementations)]
pub struct IndexedSerializer {
    delimiter: &'static [u8],
    buffer: Vec<u8>,
    map_like: bool,

    /// Value indicating whether this serializer has already serialized something. This is used to
    /// check if we need to prepend the delimiter to the next field.
    ///
    /// Note that this field cannot simply be replaced in favor of a `buffer.len() == 0` check. In
    /// case of list-like serialization the first field could be `None`, which is serialized to the
    /// empty string. In that case, a delimiter needs to be appended, but since the buffer would
    /// still be empty, no delimiter would be added.
    is_start: bool,
}

impl IndexedSerializer {
    pub fn new(delimiter: &'static str, map_like: bool) -> Self {
        IndexedSerializer {
            delimiter: delimiter.as_bytes(),
            buffer: Vec::new(),
            map_like,
            is_start: true,
        }
    }

    pub fn with_capacity(delimiter: &'static str, map_like: bool, capacity: usize) -> Self {
        IndexedSerializer {
            delimiter: delimiter.as_bytes(),
            buffer: Vec::with_capacity(capacity),
            map_like,
            is_start: true,
        }
    }

    pub fn finish(self) -> String {
        debug_assert!(std::str::from_utf8(&self.buffer[..]).is_ok());

        // We only ever put valid utf8 into the buffer

        unsafe { String::from_utf8_unchecked(self.buffer) }
    }

    fn append_integer<I: Integer>(&mut self, int: I) -> Result<(), Error> {
        if self.is_start {
            self.is_start = false;
        } else {
            self.buffer.extend_from_slice(self.delimiter);
        }

        itoa::write(&mut self.buffer, int).map_err(Error::custom)?;

        Ok(())
    }

    fn append_float<F: Floating>(&mut self, float: F) -> Result<(), Error> {
        if self.is_start {
            self.is_start = false;
        } else {
            self.buffer.extend_from_slice(self.delimiter);
        }

        dtoa::write(&mut self.buffer, float).map_err(Error::custom)?;

        Ok(())
    }

    fn append(&mut self, s: &str) -> Result<(), Error> {
        if self.is_start {
            self.is_start = false;
        } else {
            self.buffer.extend_from_slice(self.delimiter);
        }

        self.buffer.extend_from_slice(s.as_bytes());

        Ok(())
    }
}

impl<'a> Serializer for &'a mut IndexedSerializer {
    type Error = Error;
    type Ok = ();
    type SerializeMap = Impossible<(), Error>;
    type SerializeSeq = Impossible<(), Error>;
    type SerializeStruct = Self;
    type SerializeStructVariant = Impossible<(), Error>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.append(if v { "1" } else { "0" })
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.append_integer(v)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.append_integer(v)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.append_integer(v)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.append_integer(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.append_integer(v)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.append_integer(v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.append_integer(v)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.append_integer(v)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.append_float(v)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.append_float(v)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        // We don't need allocations for appending a single char
        // A buffer of size 4 is always enough to encode a char
        let mut buffer : [u8; 4]= [0; 4];
        self.append(v.encode_utf8(&mut buffer))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.append(v)
    }

    // Here we serialize bytes by base64 encoding them, so it's always valid in Geometry Dash's format
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        use base64::URL_SAFE;
        // We need to use resize instead of reserve because the base64 method for encoding takes initialized
        // slices
        let idx = self.buffer.len();
        self.buffer.resize(idx + v.len() * 4 / 3 + 4, 0);
        // This won't panic because we just allocated the right amount of data to store this
        let written = base64::encode_config_slice(v, URL_SAFE, &mut self.buffer[idx..]);
        // Shorten our vec down to just what was written
        self.buffer.resize(idx + written, 0);
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.buffer.extend_from_slice(self.delimiter);
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::Unsupported("serialize_unit"))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Error::Unsupported("serialize_unit_struct"))
    }

    fn serialize_unit_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Error::Unsupported("serialize_unit_variant"))
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(Error::Unsupported("serialize_newtype_struct"))
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self, _name: &'static str, _variant_index: u32, _variant: &'static str, _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(Error::Unsupported("serialize_newtype_variant"))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Error::Unsupported("serialize_seq"))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Error::Unsupported("serialize_tuple"))
    }

    fn serialize_tuple_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::Unsupported("serialize_tuple_struct"))
    }

    fn serialize_tuple_variant(
        self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::Unsupported("serialize_tuple_variant"))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(Error::Unsupported("serialize_map"))
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct, Self::Error> {
        // We don't store the struct name and the amount of fields doesn't matter
        Ok(self)
    }

    fn serialize_struct_variant(
        self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::Unsupported("serialize_struct_variant"))
    }

    fn collect_str<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Display,
    {
        Err(Error::Unsupported("collect_str"))
    }
}

impl<'a> SerializeStruct for &'a mut IndexedSerializer {
    type Error = Error;
    type Ok = ();

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        if self.map_like {
            self.append(key)?;
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
