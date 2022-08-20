use serde::{ Deserializer, de::{ Error as DeError, Visitor, DeserializeSeed, MapAccess, IntoDeserializer, EnumAccess, VariantAccess } };
use std::fmt::Display;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PropertyDeserError {
    #[error("Deserializing property into {ty:} isn't supported")]
    UnsupportedType { ty: &'static str },
    #[error("Can't deserialize object value into anything")]
    ObjectValueFound,
    #[error("Mismatched property type. Expected {expected:} found {found:}")]
    WrongPropType { expected: &'static str, found: &'static str },
    #[error("Expected string to have exactly 1 char")]
    ExpectedChar,
    #[error("The field type was color, but the struct has fields other than r, g, b and a")]
    ColorFieldsExhausted,
    #[error("{custom:}")]
    Custom { custom: String },
}

impl DeError for PropertyDeserError {
    fn custom<T: Display>(msg: T) -> Self { 
        PropertyDeserError::Custom { custom: format!("{}", msg) } 
    }
}

struct ColorMapper(tiled::Color, usize);

impl<'de> MapAccess<'de> for ColorMapper {
    type Error = PropertyDeserError;

    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error> {
        let ident_arr = ["red", "green", "blue", "alpha"];
        
        match ident_arr.get(self.1) {
            Some(s) => seed.deserialize(s.into_deserializer()).map(Some),
            None => Ok(None),
        }
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, Self::Error> {
        let result = match self.1 {
            0 => self.0.red,
            1 => self.0.green,
            2 => self.0.blue,
            3 => self.0.alpha,
            _ => return Err(PropertyDeserError::ColorFieldsExhausted),
        };

        self.1 += 1;
        seed.deserialize(result.into_deserializer())
    }
}

pub struct PropertyDes<'de> {
    prop: &'de tiled::PropertyValue,
}

impl<'de> PropertyDes<'de> {
    fn prop_type_str(&self) -> &'static str {
        use tiled::PropertyValue::*;

        match &self.prop {
            BoolValue(_) => "bool",
            FloatValue(_) => "float",
            IntValue(_) => "int",
            ColorValue(_) => "color",
            StringValue(_) => "string",
            FileValue(_) => "file",
            ObjectValue(_) => "object",
        }
    }

    fn parse_bool(&self) -> Result<bool, PropertyDeserError> {
        use tiled::PropertyValue::*;

        match &self.prop {
            BoolValue(x) => Ok(*x),
            _ => Err(PropertyDeserError::WrongPropType {
                expected: "bool",
                found: self.prop_type_str(),
            }),
        }
    }
    
    fn parse_f32(&self) -> Result<f32, PropertyDeserError> {
        use tiled::PropertyValue::*;

        match &self.prop {
            FloatValue(x) => Ok(*x),
            _ => Err(PropertyDeserError::WrongPropType {
                expected: "bool",
                found: self.prop_type_str(),
            }),
        }
    }
    
    fn parse_i32(&self) -> Result<i32, PropertyDeserError> {
        use tiled::PropertyValue::*;

        match &self.prop {
            IntValue(x) => Ok(*x),
            _ => Err(PropertyDeserError::WrongPropType {
                expected: "int",
                found: self.prop_type_str(),
            }),
        }
    }
    
    fn parse_str(&self) -> Result<&'de str, PropertyDeserError> {
        use tiled::PropertyValue::*;

        match &self.prop {
            StringValue(x) | FileValue(x) => Ok(x.as_str()),
            _ => Err(PropertyDeserError::WrongPropType {
                expected: "string or file path",
                found: self.prop_type_str(),
            }),
        }
    }
}

impl<'de> Deserializer<'de> for PropertyDes<'de> {
    type Error = PropertyDeserError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        use tiled::PropertyValue::*;

        match self.prop {
            BoolValue(_) => self.deserialize_bool(visitor),
            FloatValue(_) => self.deserialize_f32(visitor),
            IntValue(_) => self.deserialize_i32(visitor),
            ColorValue(_) => self.deserialize_struct("JUNK", &[], visitor),
            StringValue(_) | FileValue(_) => self.deserialize_string(visitor),
            ObjectValue(_) => Err(PropertyDeserError::ObjectValueFound),
        }
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_bool(self.parse_bool()?)
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_i32(self.parse_i32()?)
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_i32(self.parse_i32()?)
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_i32(self.parse_i32()?)
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_i32(self.parse_i32()?)
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u32(self.parse_i32()? as u32)
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u32(self.parse_i32()? as u32)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u32(self.parse_i32()? as u32)
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u32(self.parse_i32()? as u32)
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_f32(self.parse_f32()?)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_f32(self.parse_f32()?)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let s = self.parse_str()?;
        let mut char_iter = s.chars();
        match char_iter.next() {
            Some(ch) => {
                if char_iter.next().is_some() { return Err(PropertyDeserError::ExpectedChar); }

                visitor.visit_char(ch)
            },
            None => Err(PropertyDeserError::ExpectedChar),
        }
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_borrowed_str(self.parse_str()?)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_borrowed_str(self.parse_str()?)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(PropertyDeserError::UnsupportedType { ty: "bytes" })
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(PropertyDeserError::UnsupportedType { ty: "byte buffer" })
    }

    fn deserialize_option<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(PropertyDeserError::UnsupportedType { ty: "option" })
    }

    fn deserialize_unit<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(PropertyDeserError::UnsupportedType { ty: "unit" })
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(self, _name: &'static str, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(PropertyDeserError::UnsupportedType { ty: "unit struct" })
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(self, _name: &'static str, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(PropertyDeserError::UnsupportedType { ty: "newtype struct" })
    }

    fn deserialize_seq<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(PropertyDeserError::UnsupportedType { ty: "sequence" })
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(PropertyDeserError::UnsupportedType { ty: "tuple" })
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(self, _name: &'static str, _len: usize, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(PropertyDeserError::UnsupportedType { ty: "tuple struct" })
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match &self.prop {
            tiled::PropertyValue::ColorValue(col) => visitor.visit_map(ColorMapper(*col, 0)),
            _ => Err(PropertyDeserError::WrongPropType { expected: "color", found: self.prop_type_str() }),
        }
    }
    
    fn deserialize_struct<V: Visitor<'de>>(
        self, 
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_map(visitor)
    }
    
    fn deserialize_enum<V: Visitor<'de>>(
        self, 
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_enum(self.parse_str()?.into_deserializer())
    }
    
    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_borrowed_str(self.parse_str()?)
    }
    
    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_any(visitor)
    }
}

#[derive(Debug, Error)]
pub enum TilePropertyDeserError {
    #[error("Only structs can be deserialized from tiled properties")]
    OnlyStruct,
    #[error("The tile didn't have a typed specified")]
    NoType,
    #[error("Mismatched types. The tile has type {found:?}, but type {expected:?} was expected")]
    WrongType { expected: &'static str, found: String },
    #[error("Failed to parse property {name:?}. {source:}")]
    PropFail { name: String, source: PropertyDeserError },
    #[error("Tuple enum variants aren't supported")]
    TupleVariantNotSupported,
    #[error("{custom:}")]
    Custom { custom: String },
}

impl DeError for TilePropertyDeserError {
    fn custom<T: Display>(msg: T) -> Self { 
        TilePropertyDeserError::Custom { custom: format!("{}", msg) } 
    }
}

struct TilePropertyMapper<'de> {
    curr: Option<(&'de String, &'de tiled::PropertyValue)>,
    it: std::collections::hash_map::Iter<'de, String, tiled::PropertyValue>,
}

impl<'de> MapAccess<'de> for TilePropertyMapper<'de> {
    type Error = TilePropertyDeserError;

    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error> {
        self.curr = self.it.next();

        match self.curr {
            Some((s, _)) => seed.deserialize(s.as_str().into_deserializer()).map(Some),
            None => Ok(None),
        }
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, Self::Error> {
        let (name, prop) = self.curr.expect("Called after the fields were over or before `next_key_seed`");

        seed.deserialize(PropertyDes { prop })
            .map_err(|source| TilePropertyDeserError::PropFail { name: name.to_owned(), source })
    }
}

struct TilePropertyEnum<'de> {
    tile: &'de tiled::Tile<'de>,
}

impl<'de> EnumAccess<'de> for TilePropertyEnum<'de> {
    type Error = TilePropertyDeserError;
    type Variant = Self;

    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error> {
        let val = seed.deserialize(self.tile.tile_type.as_ref()
            .ok_or(TilePropertyDeserError::NoType)?
            .as_str()
            .into_deserializer()
        )?;
        Ok((val, self))
    }
}

impl<'de> VariantAccess<'de> for TilePropertyEnum<'de> {
    type Error = TilePropertyDeserError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error> 
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(TilePropertyDes { tile: self.tile })
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error> 
    where
        V: Visitor<'de>
    {
        Err(TilePropertyDeserError::TupleVariantNotSupported)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>
    {
        TilePropertyDes { tile: self.tile }.deserialize_map(visitor)
    }
}

pub struct TilePropertyDes<'de> { 
    pub tile: &'de tiled::Tile<'de>,
}

impl<'de> Deserializer<'de> for TilePropertyDes<'de> {
    type Error = TilePropertyDeserError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        // Data can only be a struct or a map
        self.deserialize_map(visitor)
    }

    fn deserialize_bool<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(TilePropertyDeserError::OnlyStruct)
    }

    fn deserialize_i8<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(TilePropertyDeserError::OnlyStruct)
    }

    fn deserialize_i16<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(TilePropertyDeserError::OnlyStruct)
    }

    fn deserialize_i32<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(TilePropertyDeserError::OnlyStruct)
    }

    fn deserialize_i64<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(TilePropertyDeserError::OnlyStruct)
    }

    fn deserialize_u8<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(TilePropertyDeserError::OnlyStruct)
    }

    fn deserialize_u16<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(TilePropertyDeserError::OnlyStruct)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(TilePropertyDeserError::OnlyStruct)
    }

    fn deserialize_u64<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(TilePropertyDeserError::OnlyStruct)
    }

    fn deserialize_f32<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(TilePropertyDeserError::OnlyStruct)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(TilePropertyDeserError::OnlyStruct)
    }

    fn deserialize_char<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(TilePropertyDeserError::OnlyStruct)
    }

    fn deserialize_str<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(TilePropertyDeserError::OnlyStruct)
    }

    fn deserialize_string<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(TilePropertyDeserError::OnlyStruct)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(TilePropertyDeserError::OnlyStruct)
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(TilePropertyDeserError::OnlyStruct)
    }

    fn deserialize_option<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(TilePropertyDeserError::OnlyStruct)
    }

    fn deserialize_unit<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(TilePropertyDeserError::OnlyStruct)
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(self, _name: &'static str, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(TilePropertyDeserError::OnlyStruct)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(self, _name: &'static str, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(TilePropertyDeserError::OnlyStruct)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(TilePropertyDeserError::OnlyStruct)
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(TilePropertyDeserError::OnlyStruct)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(self, _name: &'static str, _len: usize, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(TilePropertyDeserError::OnlyStruct)
    }

    fn deserialize_map<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(TilePropertyDeserError::OnlyStruct)
    }
    
    fn deserialize_struct<V: Visitor<'de>>(
        self, 
        name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        let ty = self.tile.tile_type.as_ref()
            .ok_or(TilePropertyDeserError::NoType)?
            .as_str();

        if ty != name { 
            return Err(TilePropertyDeserError::WrongType { 
                expected: name, 
                found: ty.to_owned(),
            });
        }

        visitor.visit_map(TilePropertyMapper {
            curr: None,
            it: self.tile.properties.iter(),
        })
    }
    
    fn deserialize_enum<V: Visitor<'de>>(
        self, 
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_enum(TilePropertyEnum { tile: &self.tile })
    }
    
    fn deserialize_identifier<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(TilePropertyDeserError::OnlyStruct)
    }
    
    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_any(visitor)
    }
}
