//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

pub trait EnumString<T> {
    fn from_str(val: &str) -> Result<T, String>;
    fn as_str(&self) -> &'static str;
}


macro_rules! enum_string_serialization {
    ($enumtype:ident $visitor:ident) => (

        struct $visitor;

        impl serde::ser::Serialize for $enumtype {
            fn serialize<__S>(&self, serializer: __S) -> Result<__S::Ok, __S::Error>
                where __S: serde::ser::Serializer
            {
                 serializer.serialize_str(&self.as_str())
            }
        }

        impl<'de> serde::de::Visitor<'de> for $visitor
        {
            type Value = $enumtype;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("enum value as string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                where E: serde::de::Error
            {
                Self::Value::from_str(value).map_err(serde::de::Error::custom)
            }

        }

        impl<'de> Deserialize<'de> for $enumtype {
            fn deserialize<D>(deserializer: D) -> Result<$enumtype, D::Error>
                where D: Deserializer<'de>
            {
                deserializer.deserialize_str($visitor)
            }
        }
    )
}
