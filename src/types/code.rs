//! One definition of what a protocol enumeration is.
//!
//! Every MQL5 enumeration on the wire is a plain integer from a set
//! MetaQuotes can grow at any build. The macro below turns a table of those
//! integers into a Rust enum that carries its own name, converts both ways,
//! and keeps an unrecognised value instead of losing it.

/// Declare an enumeration that travels as `$repr` on the wire.
///
/// The `Unknown` variant is what makes this safe against a newer terminal: a
/// value not in the table round-trips through `code()` unchanged rather than
/// being rejected or silently mapped onto something else.
macro_rules! wire_enum {
    (
        $(#[$meta:meta])*
        $name:ident : $repr:ty {
            $($(#[$variant_meta:meta])* $variant:ident = $code:expr, $label:literal;)+
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            $($(#[$variant_meta])* $variant,)+
            /// A value this crate has no name for, kept as the terminal sent
            /// it. A newer build adding a value shows up here rather than as
            /// a decode failure.
            Unknown($repr),
        }

        impl $name {
            /// The number the protocol uses.
            pub const fn code(self) -> $repr {
                match self {
                    $($name::$variant => $code,)+
                    $name::Unknown(code) => code,
                }
            }

            /// Read one off the wire. An unlisted value becomes `Unknown`.
            pub const fn from_code(code: $repr) -> Self {
                match code {
                    $($code => $name::$variant,)+
                    other => $name::Unknown(other),
                }
            }

            /// The MQL5 name, or `UNKNOWN` for a value not in the table.
            pub const fn name(self) -> &'static str {
                match self {
                    $($name::$variant => $label,)+
                    $name::Unknown(_) => "UNKNOWN",
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $name::Unknown(code) => write!(f, "UNKNOWN({code})"),
                    known => f.write_str(known.name()),
                }
            }
        }

        impl Default for $name {
            /// Whatever the protocol numbers zero, which is also what a
            /// zeroed record decodes to.
            fn default() -> Self {
                $name::from_code(0)
            }
        }

        impl From<$repr> for $name {
            fn from(code: $repr) -> Self {
                $name::from_code(code)
            }
        }

        impl From<$name> for $repr {
            fn from(value: $name) -> Self {
                value.code()
            }
        }
    };
}
