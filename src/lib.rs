use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

/// Generates a newtype UUID wrapper struct with standard trait implementations.
///
/// The generated struct wraps `uuid::Uuid` and derives `Debug`, `Clone`,
/// `PartialEq`, `Eq`, `Hash`, `serde::Serialize`, and `serde::Deserialize`.
///
/// Optionally, if the `openapi` feature is enabled on the consuming crate,
/// a `utoipa::ToSchema` derive is also emitted.
///
/// # Example
///
/// ```ignore
/// typed_id::uuid_id!(UserId);
///
/// let id = UserId::new();
/// let id2 = UserId::from_uuid(uuid::Uuid::new_v4());
/// let raw: &uuid::Uuid = id.as_uuid();
/// ```
///
/// # Generated API
///
/// - `UserId::new() -> UserId` — generates a new random v4 UUID
/// - `UserId::from_uuid(uuid: uuid::Uuid) -> UserId`
/// - `UserId::as_uuid(&self) -> &uuid::Uuid`
/// - `Default` — delegates to `new()`
///
/// # Dependencies
///
/// The consuming crate must depend on `uuid` (with the `v4` feature) and `serde`.
#[proc_macro]
pub fn uuid_id(input: TokenStream) -> TokenStream {
    let name = parse_macro_input!(input as syn::Ident);

    let expanded = quote! {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, ::serde::Serialize, ::serde::Deserialize)]
        #[cfg_attr(feature = "openapi", derive(::utoipa::ToSchema))]
        #[cfg_attr(feature = "openapi", schema(value_type = ::uuid::Uuid))]
        pub struct #name(::uuid::Uuid);

        impl Default for #name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl #name {
            pub fn new() -> Self {
                Self(::uuid::Uuid::new_v4())
            }

            pub fn from_uuid(uuid: ::uuid::Uuid) -> Self {
                Self(uuid)
            }

            pub fn as_uuid(&self) -> &::uuid::Uuid {
                &self.0
            }
        }
    };

    TokenStream::from(expanded)
}

/// Generates a newtype slug wrapper struct with validation on construction.
///
/// A slug must contain only lowercase ASCII letters, digits, and hyphens,
/// and must not start or end with a hyphen. Empty strings are rejected.
///
/// The generated struct derives `Debug`, `Clone`, `PartialEq`, `Eq`,
/// `PartialOrd`, `Ord`, `Hash`, and `serde::Serialize`. Deserialization
/// validates the string through `new()`.
///
/// # Example
///
/// ```ignore
/// typed_id::slug_id!(ProjectSlug);
///
/// let slug = ProjectSlug::new("my-project").unwrap();
/// let slug: ProjectSlug = "my-project".parse().unwrap();
/// ```
///
/// # Generated API
///
/// - `FooSlug::new(value: impl Into<String>) -> Result<FooSlug, String>`
/// - `FooSlug::as_str(&self) -> &str`
/// - `Display` — renders the inner string
/// - `TryFrom<String>` and `TryFrom<&str>` — delegates to `new()`
/// - `FromStr` — delegates to `new()`
///
/// # Dependencies
///
/// The consuming crate must depend on `serde`.
#[proc_macro]
pub fn slug_id(input: TokenStream) -> TokenStream {
    let name = parse_macro_input!(input as syn::Ident);

    let expanded = quote! {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, ::serde::Serialize)]
        #[cfg_attr(feature = "openapi", derive(::utoipa::ToSchema))]
        #[cfg_attr(feature = "openapi", schema(value_type = String))]
        #[serde(transparent)]
        pub struct #name(String);

        impl<'de> ::serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                let s = String::deserialize(deserializer)?;
                Self::new(s).map_err(::serde::de::Error::custom)
            }
        }

        impl #name {
            pub fn new(value: impl Into<String>) -> Result<Self, String> {
                let value = value.into();
                if value.is_empty() {
                    return Err("Slug cannot be empty".to_string());
                }

                if !value.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
                    return Err("Slug can only contain lowercase alphanumeric characters and hyphens".to_string());
                }

                if value.starts_with('-') || value.ends_with('-') {
                    return Err("Slug cannot start or end with a hyphen".to_string());
                }

                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl TryFrom<String> for #name {
            type Error = String;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl TryFrom<&str> for #name {
            type Error = String;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::new(value.to_string())
            }
        }

        impl std::str::FromStr for #name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::new(s.to_string())
            }
        }
    };

    TokenStream::from(expanded)
}
