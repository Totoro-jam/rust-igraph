//! Vertex, edge, and graph attribute storage.
//!
//! Mirrors the three igraph C attribute types (numeric, boolean, string)
//! with a Rust enum [`AttributeValue`]. Attribute storage lives directly
//! on [`Graph`](super::graph::Graph) as `HashMap<String, ...>` fields.

use std::fmt;

/// A single attribute value attached to a graph, vertex, or edge.
///
/// ```
/// use rust_igraph::AttributeValue;
///
/// let v = AttributeValue::Numeric(3.14);
/// assert!(v.as_f64().is_some());
/// assert!(v.as_str().is_none());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValue {
    /// Floating-point number (mirrors `IGRAPH_ATTRIBUTE_NUMERIC`).
    Numeric(f64),
    /// Boolean (mirrors `IGRAPH_ATTRIBUTE_BOOLEAN`).
    Boolean(bool),
    /// UTF-8 string (mirrors `IGRAPH_ATTRIBUTE_STRING`).
    String(String),
}

impl AttributeValue {
    /// Returns the numeric value if this is `Numeric`, otherwise `None`.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Numeric(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns the boolean value if this is `Boolean`, otherwise `None`.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Boolean(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns a string slice if this is `String`, otherwise `None`.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(v) => Some(v.as_str()),
            _ => None,
        }
    }

    /// Default value for extending attribute vectors when new
    /// vertices/edges are added and the attribute already exists.
    pub(crate) fn default_for_same_type(&self) -> Self {
        match self {
            Self::Numeric(_) => Self::Numeric(f64::NAN),
            Self::Boolean(_) => Self::Boolean(false),
            Self::String(_) => Self::String(std::string::String::new()),
        }
    }
}

impl fmt::Display for AttributeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Numeric(v) => write!(f, "{v}"),
            Self::Boolean(v) => write!(f, "{v}"),
            Self::String(v) => write!(f, "{v}"),
        }
    }
}

impl From<f64> for AttributeValue {
    fn from(v: f64) -> Self {
        Self::Numeric(v)
    }
}

impl From<bool> for AttributeValue {
    fn from(v: bool) -> Self {
        Self::Boolean(v)
    }
}

impl From<String> for AttributeValue {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl From<&str> for AttributeValue {
    fn from(v: &str) -> Self {
        Self::String(v.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors() {
        let n = AttributeValue::Numeric(2.5);
        assert!((n.as_f64().unwrap() - 2.5).abs() < f64::EPSILON);
        assert!(n.as_bool().is_none());
        assert!(n.as_str().is_none());

        let b = AttributeValue::Boolean(true);
        assert_eq!(b.as_bool(), Some(true));
        assert!(b.as_f64().is_none());

        let s = AttributeValue::String("hello".into());
        assert_eq!(s.as_str(), Some("hello"));
    }

    #[test]
    fn from_conversions() {
        let v: AttributeValue = 1.0_f64.into();
        assert_eq!(v, AttributeValue::Numeric(1.0));

        let v: AttributeValue = true.into();
        assert_eq!(v, AttributeValue::Boolean(true));

        let v: AttributeValue = "test".into();
        assert_eq!(v, AttributeValue::String("test".into()));
    }

    #[test]
    fn display() {
        assert_eq!(format!("{}", AttributeValue::Numeric(2.71)), "2.71");
        assert_eq!(format!("{}", AttributeValue::Boolean(false)), "false");
        assert_eq!(format!("{}", AttributeValue::String("hi".into())), "hi");
    }
}
