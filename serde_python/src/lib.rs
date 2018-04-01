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

impl serde::ser::Error for Error {
    fn custom<T>(_msg: T) -> Self where T: std::fmt::Display {
        unimplemented!()
    }
}

pub use ser::PyObjectSerializer;

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
