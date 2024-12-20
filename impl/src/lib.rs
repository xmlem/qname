//! This crate implements the macro for `qname` and should not be used directly.

use std::{fmt::Display, str::FromStr};

use proc_macro2::TokenStream;
use quote::quote;
use syn::LitStr;

#[doc(hidden)]
pub fn qname(item: TokenStream) -> Result<TokenStream, syn::Error> {
    // Implement your proc-macro logic here. :)
    let s: LitStr = syn::parse2(item)?;
    let _qname: QName = s
        .value()
        .parse()
        .map_err(|_| syn::Error::new(s.span(), "Invalid QName"))?;

    Ok(quote! {
        ::qname::QName::new_unchecked(#s)
    })
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QName {
    pub(crate) namespace: Option<String>,
    pub(crate) local_part: String,
    pub(crate) prefixed_name: String,
}

impl Display for QName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.prefixed_name)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Error(ErrorInner);

#[derive(Debug, Clone, Copy)]
enum ErrorInner {
    Empty,
    Start(char),
    Continue(char),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            ErrorInner::Empty => write!(f, "Invalid QName: Cannot be empty"),
            ErrorInner::Start(c) => write!(f, "Invalid QName: First char cannot be {c:?}"),
            ErrorInner::Continue(c) => write!(f, "Invalid QName: Cannot contain {c:?}"),
        }
    }
}

impl FromStr for QName {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        QName::new(s)
    }
}

impl TryFrom<&'_ str> for QName {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Error> {
        QName::new(s)
    }
}

impl QName {
    /// Attempt to parse a string as a qualified name.
    pub fn new(name: &str) -> Result<QName, Error> {
        if let Some(first_err) = first_qname_error(name) {
            return Err(first_err);
        }

        Ok(match name.split_once(":") {
            Some((ns, local)) => Self {
                namespace: Some(ns.to_string()),
                local_part: local.to_string(),
                prefixed_name: format!("{ns}:{local}"),
            },
            None => Self {
                namespace: None,
                local_part: name.to_string(),
                prefixed_name: name.to_string(),
            },
        })
    }

    /// Create a qname from a known-valid qualified name.
    ///
    /// ## Panics
    ///
    /// This function panics if the given name is not valid.
    pub fn new_unchecked(name: &str) -> QName {
        if let Some(err) = first_qname_error(name) {
            panic!("Input '{name}' is not a valid QName: {err}.");
        }

        match name.split_once(":") {
            Some((ns, local)) => Self {
                namespace: Some(ns.to_string()),
                local_part: local.to_string(),
                prefixed_name: format!("{ns}:{local}"),
            },
            None => Self {
                namespace: None,
                local_part: name.to_string(),
                prefixed_name: name.to_string(),
            },
        }
    }

    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    pub fn local_part(&self) -> &str {
        &self.local_part
    }

    pub fn prefixed_name(&self) -> &str {
        &self.prefixed_name
    }
}

pub fn is_valid_qname(input: &str) -> bool {
    first_qname_error(input).is_some()
}

fn first_qname_error(input: &str) -> Option<Error> {
    fn is_name_start_char(ch: char) -> bool {
        matches!(ch, ':'
            | 'A'..='Z'
            | '_'
            | 'a'..='z'
            | '\u{C0}'..='\u{D6}'
            | '\u{D8}'..='\u{F6}'
            | '\u{F8}'..='\u{2FF}'
            | '\u{370}'..='\u{37D}'
            | '\u{37F}'..='\u{1FFF}'
            | '\u{200C}'..='\u{200D}'
            | '\u{2070}'..='\u{218F}'
            | '\u{2C00}'..='\u{2FEF}'
            | '\u{3001}'..='\u{D7FF}'
            | '\u{F900}'..='\u{FDCF}'
            | '\u{FDF0}'..='\u{FFFD}'
            | '\u{10000}'..='\u{EFFFF}')
    }

    fn is_name_char(ch: char) -> bool {
        if is_name_start_char(ch) {
            return true;
        }

        match ch {
            '-' | '.' | '0'..='9' => return true,
            _ => {}
        }

        match ch as u32 {
            0xb7 | 0x0300..=0x036F | 0x203F..=0x2040 => true,
            _ => false,
        }
    }

    let mut chars = input.chars();
    match chars.next() {
        None => return Some(Error(ErrorInner::Empty)),
        Some(ch) => {
            if !is_name_start_char(ch) {
                return Some(Error(ErrorInner::Start(ch)));
            }
        }
    };

    chars
        .find(|&c| !is_name_char(c))
        .map(|ch| Error(ErrorInner::Continue(ch)))
}
