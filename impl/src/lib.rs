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
pub struct Error;

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Invalid QName")
    }
}

impl FromStr for QName {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        QName::new(s)
    }
}

impl QName {
    pub fn new(name: &str) -> Result<QName, Error> {
        if !is_valid_qname(name) {
            return Err(Error);
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

    pub fn new_unchecked(name: &str) -> QName {
        if !is_valid_qname(name) {
            panic!("Input '{name}' is not a valid QName.");
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
    fn is_name_start_char(ch: char) -> bool {
        match ch {
            ':' | 'A'..='Z' | '_' | 'a'..='z' => return true,
            _ => {}
        }
        match ch as u32 {
            0xC0..=0xD6
            | 0xD8..=0xF6
            | 0xF8..=0x2FF
            | 0x370..=0x37D
            | 0x37F..=0x1FFF
            | 0x200C..=0x200D
            | 0x2070..=0x218F
            | 0x2C00..=0x2FEF
            | 0x3001..=0xD7FF
            | 0xF900..=0xFDCF
            | 0xFDF0..=0xFFFD
            | 0x10000..=0xEFFFF => true,
            _ => false,
        }
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
    let is_valid = match chars.next() {
        Some(ch) => is_name_start_char(ch),
        None => false,
    };
    if !is_valid {
        return false;
    }

    chars.all(is_name_char)
}
