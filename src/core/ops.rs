//! `std::ops` trait implementations for [`Graph`].
//!
//! These provide idiomatic Rust operator syntax for common graph
//! operations:
//!
//! | Operator | Operation | Function |
//! |----------|-----------|----------|
//! | `g1 + g2` | Disjoint union | [`disjoint_union`] |
//! | `g1 \| g2` | Union (max-multiplicity) | [`union`] |
//! | `g1 & g2` | Intersection (min-multiplicity) | [`intersection`] |
//! | `g1 - g2` | Difference | [`difference`] |
//! | `!g` | Complement | [`complementer`] |
//!
//! All operators panic on error (e.g. directedness mismatch). For
//! fallible versions, use the underlying functions directly.
//!
//! [`disjoint_union`]: crate::algorithms::operators::disjoint_union::disjoint_union
//! [`union`]: crate::algorithms::operators::union::union
//! [`intersection`]: crate::algorithms::operators::intersection::intersection
//! [`difference`]: crate::algorithms::operators::difference::difference
//! [`complementer`]: crate::algorithms::operators::complementer::complementer

use std::ops::{Add, BitAnd, BitOr, Not, Sub};

use super::graph::Graph;
use crate::algorithms::operators::complementer::complementer;
use crate::algorithms::operators::difference::difference;
use crate::algorithms::operators::disjoint_union::disjoint_union;
use crate::algorithms::operators::intersection::intersection;
use crate::algorithms::operators::union::union;

/// Disjoint union: `g1 + g2` concatenates vertex sets and edge sets.
///
/// # Panics
///
/// Panics if the graphs have mismatched directedness.
///
/// # Examples
///
/// ```
/// use rust_igraph::Graph;
///
/// let a = Graph::try_from(vec![(0u32, 1)].as_slice()).unwrap();
/// let b = Graph::try_from(vec![(0u32, 1), (1, 2)].as_slice()).unwrap();
/// let c = a + b;
/// assert_eq!(c.vcount(), 5);
/// assert_eq!(c.ecount(), 3);
/// ```
impl Add for Graph {
    type Output = Graph;

    fn add(self, rhs: Graph) -> Graph {
        disjoint_union(&self, &rhs).expect("disjoint_union failed (directedness mismatch?)")
    }
}

/// Disjoint union by reference.
impl Add for &Graph {
    type Output = Graph;

    fn add(self, rhs: &Graph) -> Graph {
        disjoint_union(self, rhs).expect("disjoint_union failed (directedness mismatch?)")
    }
}

/// Union (max-multiplicity merge): `g1 | g2`.
///
/// # Panics
///
/// Panics if the graphs have mismatched directedness.
///
/// # Examples
///
/// ```
/// use rust_igraph::Graph;
///
/// let mut a = Graph::new(3, false).unwrap();
/// a.add_edge(0, 1).unwrap();
/// let mut b = Graph::new(3, false).unwrap();
/// b.add_edge(1, 2).unwrap();
/// let c = a | b;
/// assert_eq!(c.ecount(), 2);
/// ```
impl BitOr for Graph {
    type Output = Graph;

    fn bitor(self, rhs: Graph) -> Graph {
        union(&self, &rhs).expect("union failed (directedness mismatch?)")
    }
}

/// Union by reference.
impl BitOr for &Graph {
    type Output = Graph;

    fn bitor(self, rhs: &Graph) -> Graph {
        union(self, rhs).expect("union failed (directedness mismatch?)")
    }
}

/// Intersection (min-multiplicity): `g1 & g2`.
///
/// # Panics
///
/// Panics if the graphs have mismatched directedness.
///
/// # Examples
///
/// ```
/// use rust_igraph::Graph;
///
/// let mut a = Graph::new(3, false).unwrap();
/// a.add_edge(0, 1).unwrap();
/// a.add_edge(1, 2).unwrap();
/// let mut b = Graph::new(3, false).unwrap();
/// b.add_edge(1, 2).unwrap();
/// b.add_edge(2, 0).unwrap();
/// let c = a & b;
/// assert_eq!(c.ecount(), 1);
/// assert!(c.has_edge(1, 2));
/// ```
impl BitAnd for Graph {
    type Output = Graph;

    fn bitand(self, rhs: Graph) -> Graph {
        intersection(&self, &rhs).expect("intersection failed (directedness mismatch?)")
    }
}

/// Intersection by reference.
impl BitAnd for &Graph {
    type Output = Graph;

    fn bitand(self, rhs: &Graph) -> Graph {
        intersection(self, rhs).expect("intersection failed (directedness mismatch?)")
    }
}

/// Difference: `g1 - g2` (edges in `g1` not in `g2`).
///
/// # Panics
///
/// Panics if the graphs have mismatched directedness.
///
/// # Examples
///
/// ```
/// use rust_igraph::Graph;
///
/// let mut a = Graph::new(3, false).unwrap();
/// a.add_edge(0, 1).unwrap();
/// a.add_edge(1, 2).unwrap();
/// let mut b = Graph::new(3, false).unwrap();
/// b.add_edge(1, 2).unwrap();
/// let c = a - b;
/// assert_eq!(c.ecount(), 1);
/// assert!(c.has_edge(0, 1));
/// ```
impl Sub for Graph {
    type Output = Graph;

    fn sub(self, rhs: Graph) -> Graph {
        difference(&self, &rhs).expect("difference failed (directedness mismatch?)")
    }
}

/// Difference by reference.
impl Sub for &Graph {
    type Output = Graph;

    fn sub(self, rhs: &Graph) -> Graph {
        difference(self, rhs).expect("difference failed (directedness mismatch?)")
    }
}

/// Complement: `!g` (all edges not in `g`, no self-loops).
///
/// # Examples
///
/// ```
/// use rust_igraph::Graph;
///
/// // Complement of K_3 is an empty graph on 3 vertices.
/// let mut k3 = Graph::new(3, false).unwrap();
/// k3.add_edge(0, 1).unwrap();
/// k3.add_edge(1, 2).unwrap();
/// k3.add_edge(0, 2).unwrap();
/// let c = !k3;
/// assert_eq!(c.vcount(), 3);
/// assert_eq!(c.ecount(), 0);
/// ```
impl Not for Graph {
    type Output = Graph;

    fn not(self) -> Graph {
        complementer(&self, false).expect("complementer failed")
    }
}

/// Complement by reference.
impl Not for &Graph {
    type Output = Graph;

    fn not(self) -> Graph {
        complementer(self, false).expect("complementer failed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle() -> Graph {
        let mut g = Graph::new(3, false).expect("graph");
        g.add_edge(0, 1).expect("e");
        g.add_edge(1, 2).expect("e");
        g.add_edge(2, 0).expect("e");
        g
    }

    fn path3() -> Graph {
        let mut g = Graph::new(3, false).expect("graph");
        g.add_edge(0, 1).expect("e");
        g.add_edge(1, 2).expect("e");
        g
    }

    #[test]
    fn add_is_disjoint_union() {
        let a = triangle();
        let b = path3();
        let c = &a + &b;
        assert_eq!(c.vcount(), 6);
        assert_eq!(c.ecount(), 5);
    }

    #[test]
    fn bitor_is_union() {
        let a = triangle();
        let mut b = Graph::new(3, false).expect("graph");
        b.add_edge(0, 1).expect("e");
        let c = &a | &b;
        assert_eq!(c.vcount(), 3);
        assert_eq!(c.ecount(), 3);
    }

    #[test]
    fn bitand_is_intersection() {
        let a = triangle();
        let b = path3();
        let c = &a & &b;
        assert_eq!(c.vcount(), 3);
        assert_eq!(c.ecount(), 2);
    }

    #[test]
    fn sub_is_difference() {
        let a = triangle();
        let b = path3();
        let c = &a - &b;
        assert_eq!(c.vcount(), 3);
        assert_eq!(c.ecount(), 1);
    }

    #[test]
    fn not_is_complement() {
        let k3 = triangle();
        let c = !&k3;
        assert_eq!(c.vcount(), 3);
        assert_eq!(c.ecount(), 0);

        let empty = Graph::new(4, false).expect("graph");
        let full = !&empty;
        assert_eq!(full.ecount(), 6); // K_4 has 6 edges
    }

    #[test]
    fn owned_operators_work() {
        let a = triangle();
        let b = path3();
        let c = a + b;
        assert_eq!(c.vcount(), 6);
    }
}
