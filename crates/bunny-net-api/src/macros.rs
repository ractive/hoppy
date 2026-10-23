//! Crate-internal macros.

/// Declare an integer-coded response enum that never fails to deserialize.
///
/// bunny.net adds enum values without versioning the API. A closed
/// `Deserialize_repr` enum turns every such addition into a hard
/// deserialization error for the *whole* list or get response. This macro
/// generates the enum plus an extra `Unknown(repr)` variant that:
///
/// - captures any integer that is not a declared variant (`Deserialize`),
/// - serializes back to the very same integer (`Serialize`), so read-modify-write
///   commands (for example `shield waf update-rule`) echo unknown values
///   instead of dropping or rewriting them.
///
/// Generated helpers: `KNOWN`, `from_repr` (lenient), `try_from_repr` (strict,
/// `None` for undeclared values; use it to validate *user input*), `to_repr`
/// and `is_known`.
///
/// Limit: an integer that does not fit the declared repr type (for example
/// `300` for a `u8` enum) is still a deserialization error.
///
/// See `decision-log.md` ("Response enums never fail on unknown values").
#[allow(unused_macros)]
macro_rules! forward_compat_repr_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident : $repr:ty {
            $( $(#[$vmeta:meta])* $variant:ident = $value:literal ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $( $(#[$vmeta])* $variant, )+
            /// A value this client does not model yet. Preserved verbatim so
            /// it round-trips on write-back.
            Unknown($repr),
        }

        #[allow(dead_code)]
        impl $name {
            /// Every declared (non-`Unknown`) variant, in declaration order.
            pub const KNOWN: &'static [$name] = &[$( $name::$variant ),+];

            /// Lenient conversion: undeclared values become [`Self::Unknown`].
            #[must_use]
            pub const fn from_repr(n: $repr) -> Self {
                match n {
                    $( $value => $name::$variant, )+
                    other => $name::Unknown(other),
                }
            }

            /// Strict conversion for user input: `None` for undeclared values.
            #[must_use]
            pub const fn try_from_repr(n: $repr) -> Option<Self> {
                match n {
                    $( $value => Some($name::$variant), )+
                    _ => None,
                }
            }

            /// The wire integer.
            #[must_use]
            pub const fn to_repr(self) -> $repr {
                match self {
                    $( $name::$variant => $value, )+
                    $name::Unknown(n) => n,
                }
            }

            /// `false` for [`Self::Unknown`].
            #[must_use]
            pub const fn is_known(self) -> bool {
                !matches!(self, $name::Unknown(_))
            }
        }

        impl $crate::core::serde_helpers::KnownRepr for $name {
            type Repr = $repr;

            fn try_from_known(n: $repr) -> Option<Self> {
                $name::try_from_repr(n)
            }
        }

        impl ::serde::Serialize for $name {
            fn serialize<S: ::serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                ::serde::Serialize::serialize(&self.to_repr(), serializer)
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                <$repr as ::serde::Deserialize>::deserialize(deserializer).map($name::from_repr)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    forward_compat_repr_enum! {
        /// Sample enum exercising the macro.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Color: u8 {
            Red = 0,
            Green = 1,
            Blue = 7,
        }
    }

    forward_compat_repr_enum! {
        /// Signed repr.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[allow(dead_code)]
        pub enum Signed: i32 {
            Neg = -1,
            Zero = 0,
        }
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    struct Holder {
        color: Color,
        maybe: Option<Color>,
    }

    #[test]
    fn known_values_round_trip() {
        for (n, c) in [(0, Color::Red), (1, Color::Green), (7, Color::Blue)] {
            assert_eq!(serde_json::from_str::<Color>(&n.to_string()).unwrap(), c);
            assert_eq!(serde_json::to_string(&c).unwrap(), n.to_string());
            assert_eq!(c.to_repr(), n);
            assert_eq!(Color::try_from_repr(n), Some(c));
            assert!(c.is_known());
        }
        assert_eq!(Color::KNOWN.len(), 3);
    }

    #[test]
    fn unknown_value_is_preserved_and_round_trips() {
        let c: Color = serde_json::from_str("99").unwrap();
        assert_eq!(c, Color::Unknown(99));
        assert!(!c.is_known());
        assert_eq!(Color::try_from_repr(99), None);
        assert_eq!(serde_json::to_string(&c).unwrap(), "99");
    }

    #[test]
    fn unknown_value_does_not_fail_containing_struct() {
        let h: Holder = serde_json::from_str(r#"{"color": 42, "maybe": 43}"#).unwrap();
        assert_eq!(h.color, Color::Unknown(42));
        assert_eq!(h.maybe, Some(Color::Unknown(43)));
        let back = serde_json::to_value(&h).unwrap();
        assert_eq!(back, serde_json::json!({"color": 42, "maybe": 43}));
    }

    #[test]
    fn signed_repr_supports_negative_values() {
        assert_eq!(serde_json::from_str::<Signed>("-1").unwrap(), Signed::Neg);
        assert_eq!(
            serde_json::from_str::<Signed>("-5").unwrap(),
            Signed::Unknown(-5)
        );
    }
}
