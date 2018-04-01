//! Provides a Python serializer
#![deny(missing_docs)]
extern crate heck;
// extern crate itertools;
// extern crate named_type;
// #[macro_use]
// extern crate named_type_derive;
pub extern crate serde;
#[cfg(test)]
#[macro_use]
extern crate serde_derive;
#[allow(unused_imports)]
#[macro_use]
extern crate serde_python_derive;
// extern crate strum;
pub extern crate cpython;

pub use serde_python_derive::*;

/// Error value for serialization
#[derive(Debug)]
pub struct Error(PyErr);

use cpython::PyErr;

impl std::convert::From<PyErr> for Error {
    fn from(e: PyErr) -> Self {
        Error(e)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl ::std::error::Error for Error {
    // FIXME
    fn description(&self) -> &str {
        "Python Exception"
    }
}

impl serde::de::Error for Error {
    fn custom<T>(_msg: T) -> Self where T: std::fmt::Display {
        unimplemented!()
    }
}

impl serde::ser::Error for Error {
    fn custom<T>(_msg: T) -> Self where T: std::fmt::Display {
        unimplemented!()
    }
}

pub use ser::PyObjectSerializer;

mod des {
    use cpython::{exc::TypeError, ObjectProtocol, Python, PythonObjectDowncastError, PythonObjectWithCheckedDowncast, PyDict, PyErr, PyList, PyObject, PyTuple};
    use serde::de::{Deserializer, DeserializeSeed, MapAccess, SeqAccess, Visitor};
    use std;
    use super::Error;

    impl<'p> std::convert::From<PythonObjectDowncastError<'p>> for Error {
        fn from(e: PythonObjectDowncastError) -> Self {
            Error(e.into())
        }
    }

    struct PyObjectDeserializer<'p>(Python<'p>, PyObject);

    macro_rules! deserialize_values {
        { $( ($v:ident, $f:ident) ),* } => {
            $(
                fn $f<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
                    visitor.$v(self.1.extract(self.0)?)
                }
            )*
        };
    }

    impl<'de, 'a> Deserializer<'de> for &'a mut PyObjectDeserializer<'de> {
        type Error = Error;

        // Look at the input data to decide what Serde data model type to
        // deserialize as. Not all data formats are able to support this operation.
        // Formats that support `deserialize_any` are known as self-describing.
        fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where V: Visitor<'de>
        {
            unimplemented!()
        }

        deserialize_values! {
            (visit_bool, deserialize_bool),
            (visit_i8, deserialize_i8),
            (visit_i16, deserialize_i16),
            (visit_i32, deserialize_i32),
            (visit_i64, deserialize_i64),
            (visit_u8, deserialize_u8),
            (visit_u16, deserialize_u16),
            (visit_u32, deserialize_u32),
            (visit_u64, deserialize_u64),
            (visit_f32, deserialize_f32),
            (visit_f64, deserialize_f64),
            (visit_string, deserialize_string)
        }

        fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where V: Visitor<'de>
        {
            visitor.visit_bytes(self.1.extract::<Vec<u8>>(self.0)?.as_ref())
        }

        fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
            where V: Visitor<'de>
        {
            unimplemented!()
        }

        fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where V: Visitor<'de>
        {
            self.deserialize_string(visitor)
        }

        fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
            where V: Visitor<'de>
        {
            unimplemented!()
        }

        // An absent optional is represented as the JSON `null` and a present
        // optional is represented as just the contained value.
        //
        // As commented in `Serializer` implementation, this is a lossy
        // representation. For example the values `Some(())` and `None` both
        // serialize as just `null`. Unfortunately this is typically what people
        // expect when working with JSON. Other formats are encouraged to behave
        // more intelligently if possible.
        fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where V: Visitor<'de>
        {
            if self.1 == self.0.None() {
                visitor.visit_none()
            } else {
                visitor.visit_some(self)
            }
        }

        // In Serde, unit means an anonymous value containing no data.
        fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where V: Visitor<'de>
        {
            if self.1 == self.0.None() {
                visitor.visit_unit()
            } else {
                Err(PyErr::new::<TypeError, _>(self.0, "Expected null")).map_err(Error)
            }
        }

        // Unit struct means a named value containing no data.
        fn deserialize_unit_struct<V>(
            self,
            _name: &'static str,
            visitor: V
        ) -> Result<V::Value, Self::Error>
            where V: Visitor<'de>
        {
            self.deserialize_unit(visitor)
        }

        // As is done here, serializers are encouraged to treat newtype structs as
        // insignificant wrappers around the data they contain. That means not
        // parsing anything other than the contained value.
        fn deserialize_newtype_struct<V>(
            self,
            _name: &'static str,
            visitor: V
        ) -> Result<V::Value, Self::Error>
            where V: Visitor<'de>
        {
            visitor.visit_newtype_struct(self)
        }

        // Deserialization of compound types like sequences and maps happens by
        // passing the visitor an "Access" object that gives it the ability to
        // iterate through the data contained in the sequence.
        fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
            where V: Visitor<'de>
        {
            visitor.visit_seq(PythonListDeserializer(self.0, PyList::downcast_from(self.0, self.1)?, 0))
        }

        // Tuples look just like sequences in JSON. Some formats may be able to
        // represent tuples more efficiently.
        //
        // As indicated by the length parameter, the `Deserialize` implementation
        // for a tuple in the Serde data model is required to know the length of the
        // tuple before even looking at the input data.
        fn deserialize_tuple<V>(
            self,
            _len: usize,
            visitor: V
        ) -> Result<V::Value, Self::Error>
            where V: Visitor<'de>
        {
            visitor.visit_seq(PythonTupleDeserializer(self.0, PyTuple::downcast_from(self.0, self.1)?, 0))
        }

        // Tuple structs look just like sequences in JSON.
        fn deserialize_tuple_struct<V>(
            self,
            _name: &'static str,
            _len: usize,
            visitor: V
        ) -> Result<V::Value, Self::Error>
            where V: Visitor<'de>
        {
            visitor.visit_seq(PythonTupleStructDeserializer(self.0, self.1, 0))
        }

        // Much like `deserialize_seq` but calls the visitors `visit_map` method
        // with a `MapAccess` implementation, rather than the visitor's `visit_seq`
        // method with a `SeqAccess` implementation.
        fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
            where V: Visitor<'de>
        {
            visitor.visit_map(PythonDictDeserializer(self.0, PyDict::downcast_from(self.0, self.1)?, 0))
        }

        // Structs look just like maps in JSON.
        //
        // Notice the `fields` parameter - a "struct" in the Serde data model means
        // that the `Deserialize` implementation is required to know what the fields
        // are before even looking at the input data. Any key-value pairing in which
        // the fields cannot be known ahead of time is probably a map.
        fn deserialize_struct<V>(
            self,
            _name: &'static str,
            _fields: &'static [&'static str],
            visitor: V
        ) -> Result<V::Value, Self::Error>
            where V: Visitor<'de>
        {
            unimplemented!()
        }

        fn deserialize_enum<V>(
            self,
            _name: &'static str,
            _variants: &'static [&'static str],
            visitor: V
        ) -> Result<V::Value, Self::Error>
            where V: Visitor<'de>
        {
            unimplemented!()
        }

        // An identifier in Serde is the type that identifies a field of a struct or
        // the variant of an enum. In JSON, struct fields and enum variants are
        // represented as strings. In other formats they may be represented as
        // numeric indices.
        fn deserialize_identifier<V>(
            self,
            visitor: V
        ) -> Result<V::Value, Error>
            where V: Visitor<'de>
        {
            self.deserialize_str(visitor)
        }

        fn deserialize_ignored_any<V>(
            self,
            visitor: V
        ) -> Result<V::Value, Error>
            where V: Visitor<'de>
        {
            self.deserialize_any(visitor)
        }
    }

    struct PythonListDeserializer<'p>(Python<'p>, PyList, usize);

    // `SeqAccess` is provided to the `Visitor` to give it the ability to iterate
    // through elements of the sequence.
    impl<'de, 'a> SeqAccess<'de> for PythonListDeserializer<'a> {
        type Error = Error;

        fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
            where T: DeserializeSeed<'de>
        {
            if self.2 < self.1.len(self.0) {
                let (value,) = self.1.get_item(self.0, self.2).extract(self.0)?;
                let o = seed.deserialize(value).map(Some);
                self.2 += 1;
                o
            } else {
                Ok(None)
            }
        }
    }

    struct PythonTupleDeserializer<'p>(Python<'p>, PyTuple, usize);

    // `SeqAccess` is provided to the `Visitor` to give it the ability to iterate
    // through elements of the sequence.
    impl<'de, 'a> SeqAccess<'de> for PythonTupleDeserializer<'a> {
        type Error = Error;

        fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
            where T: DeserializeSeed<'de>
        {
            if self.2 < self.1.len(self.0) {
                let (value,) = self.1.get_item(self.0, self.2).extract(self.0)?;
                let o = seed.deserialize(value).map(Some);
                self.2 += 1;
                o
            } else {
                Ok(None)
            }
        }
    }

    struct PythonTupleStructDeserializer<'p>(Python<'p>, PyObject, usize);

    // `SeqAccess` is provided to the `Visitor` to give it the ability to iterate
    // through elements of the sequence.
    impl<'de, 'a> SeqAccess<'de> for PythonTupleStructDeserializer<'a> {
        type Error = Error;

        fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
            where T: DeserializeSeed<'de>
        {
            if self.2 < self.1.len(self.0)? {
                let (value,) = self.1.getattr(self.0, format!("_{}", self.2)).unwrap().extract(self.0)?;
                let o = seed.deserialize(value).map(Some);
                self.2 += 1;
                o
            } else {
                Ok(None)
            }
        }
    }

    struct PythonDictDeserializer<'p>(Python<'p>, PyDict, usize);

    // `MapAccess` is provided to the `Visitor` to give it the ability to iterate
    // through entries of the map.
    impl<'de, 'a> MapAccess<'de> for PythonDictDeserializer<'a> {
        type Error = Error;

        fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
            where K: DeserializeSeed<'de>
        {
            if self.2 < self.1.len(self.0) {
                let (key, _) = self.1.get_item(self.0, self.2).unwrap().extract(self.0)?;
                let o = seed.deserialize(key).map(Some);
                self.2 += 1;
                o
            } else {
                Ok(None)
            }
        }

        fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
            where V: DeserializeSeed<'de>
        {
            let (_, value) = self.1.get_item(self.0, self.2).unwrap().extract(self.0)?;
            let o = seed.deserialize(value)?;
            self.2 += 1;
            Ok(o)
        }
    }

    /*

    struct Enum<'a, 'de: 'a> {
        de: &'a mut Deserializer<'de>,
    }

    impl<'a, 'de> Enum<'a, 'de> {
        fn new(de: &'a mut Deserializer<'de>) -> Self {
            Enum { de: de }
        }
    }

    // `EnumAccess` is provided to the `Visitor` to give it the ability to determine
    // which variant of the enum is supposed to be deserialized.
    //
    // Note that all enum deserialization methods in Serde refer exclusively to the
    // "externally tagged" enum representation.
    impl<'de, 'a> EnumAccess<'de> for Enum<'a, 'de> {
        type Error = Error;
        type Variant = Self;

        fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
            where V: DeserializeSeed<'de>
        {
            // The `deserialize_enum` method parsed a `{` character so we are
            // currently inside of a map. The seed will be deserializing itself from
            // the key of the map.
            let val = seed.deserialize(&mut *self.de)?;
            // Parse the colon separating map key from value.
            if self.de.next_char()? == ':' {
                Ok((val, self))
            } else {
                Err(Error::ExpectedMapColon)
            }
        }
    }

    // `VariantAccess` is provided to the `Visitor` to give it the ability to see
    // the content of the single variant that it decided to deserialize.
    impl<'de, 'a> VariantAccess<'de> for Enum<'a, 'de> {
        type Error = Error;

        // If the `Visitor` expected this variant to be a unit variant, the input
        // should have been the plain string case handled in `deserialize_enum`.
        fn unit_variant(self) -> Result<()> {
            Err(Error::ExpectedString)
        }

        // Newtype variants are represented in JSON as `{ NAME: VALUE }` so
        // deserialize the value here.
        fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
            where T: DeserializeSeed<'de>
        {
            seed.deserialize(self.de)
        }

        // Tuple variants are represented in JSON as `{ NAME: [DATA...] }` so
        // deserialize the sequence of data here.
        fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
            where V: Visitor<'de>
        {
            de::Deserializer::deserialize_seq(self.de, visitor)
        }

        // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }` so
        // deserialize the inner map here.
        fn struct_variant<V>(
            self,
            _fields: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value>
            where V: Visitor<'de>
        {
            de::Deserializer::deserialize_map(self.de, visitor)
        }
    }

    */
}

mod ser {
    pub use cpython;
    pub use serde;
    use std;
    use super::Error;
    use cpython::{ObjectProtocol, Python, PythonObject, PyDict, PyList, PyObject, PyTuple, ToPyObject};
    use heck::ShoutySnakeCase;
    // use itertools::Itertools;
    // use named_type::NamedType;
    use serde::ser::*;
    // use std::fmt::Debug;
    // use strum::IntoEnumIterator;
    /// Serializer that can transform items into python objects
    pub struct PyObjectSerializer<'p>(Python<'p>);

    impl<'p> PyObjectSerializer<'p> {
        /// Used to create a new Python Serializer
        pub fn new(p: Python<'p>) -> Self {
            PyObjectSerializer(p)
        }
    }

    macro_rules! serialize_values {
        { $( ($t:ty, $f:ident) ),* } => {
            $(
                fn $f(self, v: $t) -> Result<Self::Ok, Self::Error> {
                    self.serialize_value(v)
                }
            )*
        };
    }

    impl<'p> PyObjectSerializer<'p> {
        fn serialize_value<T, E>(self, v: T) -> Result<PyObject, E> where T: ToPyObject {
            Ok(v.into_py_object(self.0).into_object())
        }
    }

    impl<'p> Serializer for PyObjectSerializer<'p> {
        type Ok = PyObject;
        type Error = Error;
        type SerializeSeq = PythonVecSerializer<'p>;
        type SerializeTuple = PythonVecSerializer<'p>;
        type SerializeTupleStruct = PythonStructSerializer<'p>;
        type SerializeTupleVariant = PythonTupleVariantSerializer<'p>;
        type SerializeMap = PythonDictSerializer<'p>;
        type SerializeStruct = PythonStructSerializer<'p>;
        type SerializeStructVariant = PythonStructVariantSerializer<'p>;


        serialize_values! {
            (bool, serialize_bool),
            (i8, serialize_i8),
            (i16, serialize_i16),
            (i32, serialize_i32),
            (i64, serialize_i64),
            (u8, serialize_u8),
            (u16, serialize_u16),
            (u32, serialize_u32),
            (u64, serialize_u64),
            (f32, serialize_f32),
            (f64, serialize_f64),
            (&str, serialize_str),
            (&[u8], serialize_bytes)
        }
        fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
            self.serialize_str(&v.to_string())
        }
        fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
            Ok(self.0.None())
        }
        fn serialize_some<T: ?Sized>(
            self, 
            value: &T
        ) -> Result<Self::Ok, Self::Error>
        where
            T: Serialize {
                value.serialize(self)
            }
        fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
            Ok(self.0.None())
        }
        fn serialize_unit_struct(
            self, 
            _name: &'static str
        ) -> Result<Self::Ok, Self::Error> {
            self.serialize_unit()
        }
        fn serialize_unit_variant(
            self, 
            _name: &'static str, 
            _variant_index: u32, 
            variant: &'static str
        ) -> Result<Self::Ok, Self::Error> {
            // FIXME
            self.serialize_str(&variant.to_shouty_snake_case())
            // Ok(type_incremental_enum(self.0, name, variant, variant_index)?.getattr(self.0, variant)?)
        }
        fn serialize_newtype_struct<T: ?Sized>(
            self, 
            _name: &'static str, 
            value: &T
        ) -> Result<Self::Ok, Self::Error>
        where
            T: Serialize {
                value.serialize(self)
            }
        fn serialize_newtype_variant<T: ?Sized>(
            self, 
            _name: &'static str, 
            _variant_index: u32, 
            _variant: &'static str, 
            _value: &T
        ) -> Result<Self::Ok, Self::Error>
        where
            T: Serialize {
                // FIXME
                unimplemented!()
            }
        fn serialize_seq(
            self, 
            len: Option<usize>
        ) -> Result<Self::SerializeSeq, Self::Error> {
            if let Some(len) = len {
                Ok(PythonVecSerializer(self.0, Vec::with_capacity(len)))
            } else {
                Ok(PythonVecSerializer(self.0, Vec::new()))
            }
        }
        fn serialize_tuple(
            self, 
            len: usize
        ) -> Result<Self::SerializeTuple, Self::Error> {
            Ok(PythonVecSerializer(self.0, Vec::with_capacity(len)))
        }
        fn serialize_tuple_struct(
            self, 
            name: &'static str, 
            _len: usize
        ) -> Result<Self::SerializeTupleStruct, Self::Error> {
            PythonStructSerializer::new(self.0, name)
        }
        fn serialize_tuple_variant(
            self, 
            _name: &'static str, 
            _variant_index: u32, 
            _variant: &'static str, 
            _len: usize
        ) -> Result<Self::SerializeTupleVariant, Self::Error> {
            // FIXME
            unimplemented!()
        }
        fn serialize_map(
            self, 
            _len: Option<usize>
        ) -> Result<Self::SerializeMap, Self::Error> {
            Ok(PythonDictSerializer(self.0, PyDict::new(self.0), None))
        }
        fn serialize_struct(
            self, 
            name: &'static str, 
            _len: usize
        ) -> Result<Self::SerializeStruct, Self::Error> {
            PythonStructSerializer::new(self.0, name)
        }
        fn serialize_struct_variant(
            self, 
            _name: &'static str, 
            _variant_index: u32, 
            _variant: &'static str, 
            _len: usize
        ) -> Result<Self::SerializeStructVariant, Self::Error> {
            // FIXME
            unimplemented!()
        }
    }

    /// Serializer for dict-like objects
    pub struct PythonDictSerializer<'p>(Python<'p>, PyDict, Option<PyObject>);

    impl<'p> SerializeMap for PythonDictSerializer<'p> {
        type Ok = <PyObjectSerializer<'p> as Serializer>::Ok;
        type Error = Error;

        fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
            where T: ?Sized + Serialize
        {
            self.2 = Some(key.serialize(PyObjectSerializer(self.0))?);
            Ok(())
        }

        fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
            where T: ?Sized + Serialize
        {
            let mut key = None;
            std::mem::swap(&mut key, &mut self.2);
            self.1.set_item(self.0, key, value.serialize(PyObjectSerializer(self.0))?).map_err(Error)
        }

        fn end(self) -> Result<Self::Ok, Self::Error> {
            Ok(self.1.into_object())
        }
    }

    /*
    fn type_incremental_enum<'p>(p: Python<'p>, name: &str, variant: &str, variant_index: u32) -> Result<PyObject, Error> {
        let o = p.eval(
            &format!(
                "locals().get(\"{}\", type(\"{}\",(enum.Enum,),{{}})",
                name,
                name,
            ),
            None,
            None
        )?;
        o.setattr(p, variant, variant_index)?;
        Ok(o)
    }

    fn type_enum<'p, T>(p: Python<'p>) -> Result<PyObject, Error> where T: Debug+IntoEnumIterator+NamedType, <T as IntoEnumIterator>::Iterator: Iterator<Item=T> {
        let name = T::short_type_name();
        p.eval(
            &format!(
                "locals().get(\"{}\", type(\"{}\",(),{{{}}})",
                name,
                name,
                T::iter().map(|x| format!("{}: {:?}", format!("{:?}", x).to_shouty_snake_case(), std::mem::discriminant(&x))).join(",")
            ),
            None,
            None
        ).map_err(Error)
    }
    */

    /// Serializer for structs
    pub struct PythonStructSerializer<'p>(Python<'p>, PyObject, usize);

    impl<'p> PythonStructSerializer<'p> {
        fn new(p: Python<'p>, name: &str) -> Result<Self, Error> {
            Ok(PythonStructSerializer(p, p.eval(&format!("locals().get(\"{}\", type(\"{}\",(),{{}}))", name, name), None, None).map_err(Error)?, 0))
        }
    }

    impl<'p> SerializeStruct for PythonStructSerializer<'p> {
        type Ok = <PyObjectSerializer<'p> as Serializer>::Ok;
        type Error = Error;

        fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
            where T: ?Sized + Serialize
        {
            self.1.setattr(self.0, key, value.serialize(PyObjectSerializer(self.0))?).map_err(Error)
        }

        fn end(self) -> Result<Self::Ok, Self::Error> {
            Ok(self.1)
        }
    }

    impl<'p> SerializeTupleStruct for PythonStructSerializer<'p> {
        type Ok = <PyObjectSerializer<'p> as Serializer>::Ok;
        type Error = <PyObjectSerializer<'p> as Serializer>::Error;

        fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
            where T: ?Sized + Serialize
        {
            let o = self.1.setattr(self.0, format!("_{}", self.2), value.serialize(PyObjectSerializer(self.0))?).map_err(Error);
            self.2 += 1;
            o
        }

        fn end(self) -> Result<Self::Ok, Self::Error> {
            Ok(self.1)
        }
    }

    /// Serializer for struct variants (not implemented)
    pub struct PythonStructVariantSerializer<'p>(Python<'p>);

    impl<'p> SerializeStructVariant for PythonStructVariantSerializer<'p> {
        type Ok = <PyObjectSerializer<'p> as Serializer>::Ok;
        type Error = Error;

        fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<(), Self::Error>
            where T: ?Sized + Serialize
        {
            // FIXME
            unimplemented!()
        }

        fn end(self) -> Result<Self::Ok, Self::Error> {
            // FIXME
            unimplemented!()
        }
    }

    /// Serializer for tuple variants (not implemented)
    pub struct PythonTupleVariantSerializer<'p>(Python<'p>);

    impl<'p> SerializeTupleVariant for PythonTupleVariantSerializer<'p> {
        type Ok = <PyObjectSerializer<'p> as Serializer>::Ok;
        type Error = <PyObjectSerializer<'p> as Serializer>::Error;

        fn serialize_field<T>(&mut self, _value: &T) -> Result<(), Self::Error>
            where T: ?Sized + Serialize
        {
            // FIXME
            unimplemented!()
        }

        fn end(self) -> Result<Self::Ok, Self::Error> {
            // FIXME
            unimplemented!()
        }
    }

    /// Serializer for vec-like elements (tuple and lists)
    pub struct PythonVecSerializer<'p>(Python<'p>, Vec<PyObject>);

    impl<'p> SerializeSeq for PythonVecSerializer<'p> {
        type Ok = <PyObjectSerializer<'p> as Serializer>::Ok;
        type Error = <PyObjectSerializer<'p> as Serializer>::Error;

        // Serialize a single element of the sequence.
        fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
            where T: ?Sized + Serialize
        {
            self.1.push(value.serialize(PyObjectSerializer(self.0))?);
            Ok(())
        }

        // Close the sequence.
        fn end(self) -> Result<Self::Ok, Self::Error> {
            Ok(PyList::new(self.0, self.1.as_slice()).into_object())
        }
    }

    impl<'p> SerializeTuple for PythonVecSerializer<'p> {
        type Ok = <PyObjectSerializer<'p> as Serializer>::Ok;
        type Error = <PyObjectSerializer<'p> as Serializer>::Error;

        fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
            where T: ?Sized + Serialize
        {
            self.1.push(value.serialize(PyObjectSerializer(self.0))?);
            Ok(())
        }

        fn end(self) -> Result<Self::Ok, Self::Error> {
            Ok(PyTuple::new(self.0, self.1.as_slice()).into_object())
        }
    }
}
