#[doc(inline)]
pub use qname_macro::qname;

#[doc(inline)]
pub use qname_impl::{Error, QName};

#[cfg(test)]
mod tests {
    use super::QName;

    #[test]
    fn qname() {
        assert!(QName::new("9").is_err());
        assert!(QName::new("").is_err());
        assert!(QName::new("\n").is_err());
        assert!(QName::new("9\n").is_err());
        assert!(QName::new("\n9").is_err());
    }
}
