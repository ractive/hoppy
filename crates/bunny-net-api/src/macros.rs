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

/// Attach the spec label table to a [`forward_compat_repr_enum!`] enum.
///
/// Generates, from ONE table of `Variant => "SpecName"` pairs:
///
/// - `NAMES`: every declared variant with its spec name (also drives help-text
///   sync tests),
/// - `name()` and `describe_all()` (`"1=Allow, 2=Block, ..."`),
/// - `Display` (the spec name, or `Unknown(n)`),
/// - `FromStr` accepting a spec name (case-insensitive) or a declared wire
///   number. Labels that older hoppy versions used for a *different* number are
///   deliberately not accepted: an alias would silently flip behaviour.
#[allow(unused_macros)]
macro_rules! repr_enum_names {
    (
        $name:ident : $repr:ty, $what:literal, [
            $( $variant:ident => $label:literal ),+ $(,)?
        ]
    ) => {
        impl $name {
            /// Spec name for every declared variant. Single source of truth for
            /// `Display`, `FromStr` and the CLI help text.
            pub const NAMES: &'static [($name, &'static str)] = &[
                $( ($name::$variant, $label) ),+
            ];

            /// The spec name, or `None` for [`Self::Unknown`].
            #[must_use]
            pub fn name(self) -> Option<&'static str> {
                Self::NAMES
                    .iter()
                    .find_map(|(v, name)| (*v == self).then_some(*name))
            }

            /// `"1=Allow, 2=Block, ..."`, for help and error text.
            #[must_use]
            pub fn describe_all() -> String {
                Self::NAMES
                    .iter()
                    .map(|(v, name)| format!("{}={name}", v.to_repr()))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self.name() {
                    Some(name) => f.write_str(name),
                    None => write!(f, "Unknown({})", self.to_repr()),
                }
            }
        }

        /// Parses a spec name (case-insensitive) or a declared wire number.
        impl ::std::str::FromStr for $name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let t = s.trim();
                let found = if let Ok(n) = t.parse::<$repr>() {
                    Self::try_from_repr(n)
                } else {
                    Self::NAMES
                        .iter()
                        .find_map(|(v, name)| name.eq_ignore_ascii_case(t).then_some(*v))
                };
                found.ok_or_else(|| {
                    format!(
                        "invalid {} '{s}': expected a name or number ({})",
                        $what,
                        Self::describe_all()
                    )
                })
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

    // -----------------------------------------------------------------------
    // Regression guard (iter-87 decision "Response enums never fail on unknown
    // values"). Scans the crate sources so a new enum cannot silently opt out.
    // -----------------------------------------------------------------------

    fn rust_sources(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                rust_sources(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }

    #[test]
    fn no_closed_integer_enums_in_sources() {
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut files = Vec::new();
        rust_sources(&src, &mut files);
        for f in files {
            if f.ends_with("macros.rs") {
                continue;
            }
            let text = std::fs::read_to_string(&f).unwrap();
            for needle in ["Deserialize_repr", "serde_repr"] {
                assert!(
                    !text.contains(needle),
                    "{} uses `{needle}`: declare integer enums with \
                     forward_compat_repr_enum! so unknown values round-trip",
                    f.display()
                );
            }
        }
    }

    /// String-coded enums that derive `Deserialize` need a `#[serde(other)]`
    /// fallback, unless they are request-only.
    #[test]
    fn string_enums_have_serde_other_fallback() {
        // Request-only: never parsed from an API response.
        const REQUEST_ONLY: &[&str] = &["Granularity", "AuditLogOrder"];
        // Enums that can be sent in request bodies: their `Unknown` fallback
        // must refuse to serialise rather than emit a bogus string.
        const SENT_IN_REQUESTS: &[&str] = &[
            "RuntimeType",
            "EndpointType",
            "ImagePullPolicy",
            "Protocol",
            "AnycastIpProtocolVersion",
            "RegistryType",
            "LogForwardingType",
            "SyslogFormat",
            "Authorization",
        ];
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut files = Vec::new();
        rust_sources(&src, &mut files);
        for f in files {
            if f.ends_with("macros.rs") {
                continue;
            }
            let text = std::fs::read_to_string(&f).unwrap();
            // Only look at code before the file's unit-test module.
            let code = text.split("#[cfg(test)]").next().unwrap();
            let lines: Vec<&str> = code.lines().collect();
            let mut i = 0;
            while i < lines.len() {
                let l = lines[i];
                if let Some(rest) = l.strip_prefix("pub enum ") {
                    let name = rest.split(|c: char| !c.is_alphanumeric()).next().unwrap();
                    let mut j = i;
                    while j < lines.len() && lines[j] != "}" {
                        j += 1;
                    }
                    let body = lines[i..j].join("\n");
                    // Attributes directly above the enum.
                    let mut k = i;
                    let mut attrs = String::new();
                    while k > 0
                        && (lines[k - 1].starts_with("#[") || lines[k - 1].starts_with("///"))
                    {
                        k -= 1;
                        attrs.push_str(lines[k]);
                    }
                    let derives_de = attrs.contains("Deserialize");
                    if derives_de && !REQUEST_ONLY.contains(&name) && !body.contains("serde(other")
                    {
                        panic!(
                            "{}: enum {name} derives Deserialize without a \
                             #[serde(other)] fallback; add one or list it as request-only",
                            f.display()
                        );
                    }
                    if SENT_IN_REQUESTS.contains(&name)
                        && !body.contains("serde(other, skip_serializing)")
                    {
                        panic!(
                            "{}: enum {name} is sent in requests; its fallback must be \
                             #[serde(other, skip_serializing)]",
                            f.display()
                        );
                    }
                    i = j;
                }
                i += 1;
            }
        }
    }
}
