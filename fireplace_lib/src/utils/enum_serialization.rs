//! Automatic De/Serialization implementation for enums

/// Wrapping an enum in `enum_str` will implement `De/Serialize`.
///
/// The implementation does not follow serde's derived Implementation
/// yielding numbers for every variant, but instead creates and matches the
/// variant names as strings.
///
/// # Example
///
/// ```
/// use serde_json;
///
/// enum_str!(pub enum MyConfigEnum {
///     Enabled,
///     Disabled
/// });
///
/// assert_eq!(serde_json::from_str("Enabled"), MyConfigEnum::Enabled);
/// assert_eq!(&*serde_json::to_string(MyConfigEnum::Enabled), "Enabled");
/// ```
#[macro_export]
macro_rules! enum_str {
    ( pub enum $name:ident { $($variant:ident, )* }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
        #[allow(missing_docs)]
        pub enum $name {
            $($variant,)*
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: ::serde::Serializer,
            {
                // Serialize the enum as a string.
                serializer.serialize_str(match *self {
                    $( $name::$variant => stringify!($variant), )*
                })
            }
        }

        impl ::serde::Deserialize for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: ::serde::Deserializer,
            {
                struct Visitor;

                impl ::serde::de::Visitor for Visitor {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                        formatter.write_str("a valid variant for $name")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<$name, E>
                        where E: ::serde::de::Error,
                    {
                        match value {
                            $( stringify!($variant) => Ok($name::$variant), )*
                            x => Err(E::invalid_value(
                            ::serde::de::Unexpected::Str(x), &self)),
                        }
                    }
                }

                // Deserialize the enum from a string.
                deserializer.deserialize_str(Visitor)
            }
        }
    };
    ( enum $name:ident { $($variant:ident, )* }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
        enum $name {
            $($variant,)*
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: ::serde::Serializer,
            {
                // Serialize the enum as a string.
                serializer.serialize_str(match *self {
                    $( $name::$variant => stringify!($variant), )*
                })
            }
        }

        impl ::serde::Deserialize for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: ::serde::Deserializer,
            {
                struct Visitor;

                impl ::serde::de::Visitor for Visitor {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                        formatter.write_str("a valid variant for $name")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<$name, E>
                        where E: ::serde::de::Error,
                    {
                        match value {
                            $( stringify!($variant) => Ok($name::$variant), )*
                            x => Err(E::invalid_value(
                            ::serde::de::Unexpected::Str(x), &self)),
                        }
                    }
                }

                // Deserialize the enum from a string.
                deserializer.deserialize_str(Visitor)
            }
        }
    }
}
