//! Famous named graphs (ALGO-CN-020).
//!
//! Counterpart of `igraph_famous()` in
//! `references/igraph/src/constructors/famous.c:423-498`. Given a
//! case-insensitive name, returns the canonical labelled copy of one of
//! 31 small graphs that appear repeatedly in graph-theory literature
//! (Bull, Chvátal, Coxeter, Cubical, Diamond, Dodecahedron, Folkman,
//! Franklin, Frucht, Grötzsch, Heawood, Herschel, House, `HouseX`,
//! Icosahedron, Krackhardt Kite, Levi, `McGee`, Meredith, Noperfectmatching,
//! Nonline, Octahedron, Petersen, Robertson, Smallestcyclicgroup,
//! Tetrahedron, Thomassen, Tutte, Uniquely3colorable, Walther, Zachary).
//!
//! Aliases follow upstream: `dodecahedral ≡ dodecahedron`,
//! `grotzsch ≡ groetzsch`, `icosahedral ≡ icosahedron`,
//! `octahedral ≡ octahedron`, `tetrahedral ≡ tetrahedron`. Every famous
//! graph is undirected.
//!
//! The edge tables are transliterated byte-for-byte from
//! `famous.c:26-249` — each row holds `[vcount, ecount, directed_flag,
//! edge0_a, edge0_b, …]` so the data layout matches `igraph_i_famous()`
//! and the conformance test compares the raw ordered edge list.

use crate::core::{Graph, IgraphError, IgraphResult};

/// Build a famous named graph by name (case-insensitive).
///
/// All 31 graphs ship as undirected. `Bull`, `Chvatal`, `Coxeter`,
/// `Cubical`, `Diamond`, `Dodecahedral`/`Dodecahedron`, `Folkman`,
/// `Franklin`, `Frucht`, `Grotzsch`/`Groetzsch`, `Heawood`, `Herschel`,
/// `House`, `HouseX`, `Icosahedral`/`Icosahedron`, `Krackhardt_Kite`,
/// `Levi`, `McGee`, `Meredith`, `Noperfectmatching`, `Nonline`,
/// `Octahedral`/`Octahedron`, `Petersen`, `Robertson`,
/// `Smallestcyclicgroup`, `Tetrahedral`/`Tetrahedron`, `Thomassen`,
/// `Tutte`, `Uniquely3colorable`, `Walther`, `Zachary`.
///
/// # Errors
///
/// * [`IgraphError::InvalidArgument`] — `name` does not match any
///   supported graph (case-insensitive, ASCII).
///
/// # Examples
///
/// ```
/// use rust_igraph::famous;
///
/// let petersen = famous("Petersen").unwrap();
/// assert_eq!(petersen.vcount(), 10);
/// assert_eq!(petersen.ecount(), 15);
///
/// // Case-insensitive.
/// let zachary = famous("zachary").unwrap();
/// assert_eq!(zachary.vcount(), 34);
/// assert_eq!(zachary.ecount(), 78);
/// ```
pub fn famous(name: &str) -> IgraphResult<Graph> {
    for &(candidate, data) in TABLE {
        if name.eq_ignore_ascii_case(candidate) {
            return materialize(data);
        }
    }
    Err(IgraphError::InvalidArgument(format!(
        "famous: '{name}' is not a known graph; see famous_names() for the list"
    )))
}

/// Canonical (lowercase) names of every famous graph supported by
/// [`famous`]. Aliases are not duplicated here — each graph appears once
/// under its primary name. `famous` itself still accepts every alias.
///
/// ```
/// use rust_igraph::famous_names;
///
/// let names = famous_names();
/// assert!(names.contains(&"petersen"));
/// assert!(names.contains(&"zachary"));
/// ```
#[must_use]
pub fn famous_names() -> &'static [&'static str] {
    &[
        "bull",
        "chvatal",
        "coxeter",
        "cubical",
        "diamond",
        "dodecahedron",
        "folkman",
        "franklin",
        "frucht",
        "grotzsch",
        "heawood",
        "herschel",
        "house",
        "housex",
        "icosahedron",
        "krackhardt_kite",
        "levi",
        "mcgee",
        "meredith",
        "noperfectmatching",
        "nonline",
        "octahedron",
        "petersen",
        "robertson",
        "smallestcyclicgroup",
        "tetrahedron",
        "thomassen",
        "tutte",
        "uniquely3colorable",
        "walther",
        "zachary",
    ]
}

fn materialize(data: &[u32]) -> IgraphResult<Graph> {
    let vcount = data[0];
    let ecount = data[1] as usize;
    let directed = data[2] != 0;

    let expected_len =
        3usize
            .checked_add(ecount.checked_mul(2).ok_or_else(|| {
                IgraphError::InvalidArgument("famous: edge buffer overflow".into())
            })?)
            .ok_or_else(|| IgraphError::InvalidArgument("famous: edge buffer overflow".into()))?;
    if data.len() != expected_len {
        return Err(IgraphError::InvalidArgument(format!(
            "famous: data length mismatch (expected {expected_len}, got {})",
            data.len()
        )));
    }

    let mut g = Graph::new(vcount, directed)?;
    let edges: Vec<(u32, u32)> = data[3..]
        .chunks_exact(2)
        .map(|pair| (pair[0], pair[1]))
        .collect();
    g.add_edges(edges)?;
    Ok(g)
}

// Lookup table: every accepted name (canonical + aliases) maps to its
// edge data array. Sorted by canonical name; aliases sit next to the
// graph they alias.
#[allow(clippy::type_complexity)]
const TABLE: &[(&str, &[u32])] = &[
    ("bull", DATA_BULL),
    ("chvatal", DATA_CHVATAL),
    ("coxeter", DATA_COXETER),
    ("cubical", DATA_CUBICAL),
    ("diamond", DATA_DIAMOND),
    ("dodecahedral", DATA_DODECAHEDRON),
    ("dodecahedron", DATA_DODECAHEDRON),
    ("folkman", DATA_FOLKMAN),
    ("franklin", DATA_FRANKLIN),
    ("frucht", DATA_FRUCHT),
    ("groetzsch", DATA_GROTZSCH),
    ("grotzsch", DATA_GROTZSCH),
    ("heawood", DATA_HEAWOOD),
    ("herschel", DATA_HERSCHEL),
    ("house", DATA_HOUSE),
    ("housex", DATA_HOUSEX),
    ("icosahedral", DATA_ICOSAHEDRON),
    ("icosahedron", DATA_ICOSAHEDRON),
    ("krackhardt_kite", DATA_KRACKHARDT_KITE),
    ("levi", DATA_LEVI),
    ("mcgee", DATA_MCGEE),
    ("meredith", DATA_MEREDITH),
    ("noperfectmatching", DATA_NOPERFECTMATCHING),
    ("nonline", DATA_NONLINE),
    ("octahedral", DATA_OCTAHEDRON),
    ("octahedron", DATA_OCTAHEDRON),
    ("petersen", DATA_PETERSEN),
    ("robertson", DATA_ROBERTSON),
    ("smallestcyclicgroup", DATA_SMALLESTCYCLICGROUP),
    ("tetrahedral", DATA_TETRAHEDRON),
    ("tetrahedron", DATA_TETRAHEDRON),
    ("thomassen", DATA_THOMASSEN),
    ("tutte", DATA_TUTTE),
    ("uniquely3colorable", DATA_UNIQUELY3COLORABLE),
    ("walther", DATA_WALTHER),
    ("zachary", DATA_ZACHARY),
];

// =============================================================================
// Edge tables — transliterated byte-for-byte from
// references/igraph/src/constructors/famous.c:26-249.
// Each row holds [vcount, ecount, directed_flag, edge0_a, edge0_b, …].
// =============================================================================

#[rustfmt::skip]
const DATA_BULL: &[u32] = &[
    5, 5, 0,
    0, 1, 0, 2, 1, 2, 1, 3, 2, 4,
];

#[rustfmt::skip]
const DATA_CHVATAL: &[u32] = &[
    12, 24, 0,
    5, 6, 6, 7, 7, 8, 8, 9, 5, 9, 4, 5, 4, 8, 2, 8, 2, 6, 0, 6, 0, 9, 3, 9, 3, 7,
    1, 7, 1, 5, 1, 10, 4, 10, 4, 11, 2, 11, 0, 10, 0, 11, 3, 11, 3, 10, 1, 2,
];

#[rustfmt::skip]
const DATA_COXETER: &[u32] = &[
    28, 42, 0,
    0, 1, 0, 2, 0, 7, 1, 4, 1, 13, 2, 3, 2, 8, 3, 6, 3, 9, 4, 5, 4, 12, 5, 6, 5,
    11, 6, 10, 7, 19, 7, 24, 8, 20, 8, 23, 9, 14, 9, 22, 10, 15, 10, 21, 11, 16,
    11, 27, 12, 17, 12, 26, 13, 18, 13, 25, 14, 17, 14, 18, 15, 18, 15, 19, 16, 19,
    16, 20, 17, 20, 21, 23, 21, 26, 22, 24, 22, 27, 23, 25, 24, 26, 25, 27,
];

#[rustfmt::skip]
const DATA_CUBICAL: &[u32] = &[
    8, 12, 0,
    0, 1, 1, 2, 2, 3, 0, 3, 4, 5, 5, 6, 6, 7, 4, 7, 0, 4, 1, 5, 2, 6, 3, 7,
];

#[rustfmt::skip]
const DATA_DIAMOND: &[u32] = &[
    4, 5, 0,
    0, 1, 0, 2, 1, 2, 1, 3, 2, 3,
];

#[rustfmt::skip]
const DATA_DODECAHEDRON: &[u32] = &[
    20, 30, 0,
    0, 1, 0, 4, 0, 5, 1, 2, 1, 6, 2, 3, 2, 7, 3, 4, 3, 8, 4, 9, 5, 10, 5, 11, 6,
    10, 6, 14, 7, 13, 7, 14, 8, 12, 8, 13, 9, 11, 9, 12, 10, 15, 11, 16, 12, 17,
    13, 18, 14, 19, 15, 16, 15, 19, 16, 17, 17, 18, 18, 19,
];

#[rustfmt::skip]
const DATA_FOLKMAN: &[u32] = &[
    20, 40, 0,
    0, 5, 0, 8, 0, 10, 0, 13, 1, 7, 1, 9, 1, 12, 1, 14, 2, 6, 2, 8, 2, 11, 2, 13,
    3, 5, 3, 7, 3, 10, 3, 12, 4, 6, 4, 9, 4, 11, 4, 14, 5, 15, 5, 19, 6, 15, 6, 16,
    7, 16, 7, 17, 8, 17, 8, 18, 9, 18, 9, 19, 10, 15, 10, 19, 11, 15, 11, 16, 12,
    16, 12, 17, 13, 17, 13, 18, 14, 18, 14, 19,
];

#[rustfmt::skip]
const DATA_FRANKLIN: &[u32] = &[
    12, 18, 0,
    0, 1, 0, 2, 0, 6, 1, 3, 1, 7, 2, 4, 2, 10, 3, 5, 3, 11, 4, 5, 4, 6, 5, 7, 6, 8,
    7, 9, 8, 9, 8, 11, 9, 10, 10, 11,
];

#[rustfmt::skip]
const DATA_FRUCHT: &[u32] = &[
    12, 18, 0,
    0, 1, 0, 2, 0, 11, 1, 3, 1, 6, 2, 5, 2, 10, 3, 4, 3, 6, 4, 8, 4, 11, 5, 9, 5,
    10, 6, 7, 7, 8, 7, 9, 8, 9, 10, 11,
];

#[rustfmt::skip]
const DATA_GROTZSCH: &[u32] = &[
    11, 20, 0,
    0, 1, 0, 2, 0, 7, 0, 10, 1, 3, 1, 6, 1, 9, 2, 4, 2, 6, 2, 8, 3, 4, 3, 8, 3, 10,
    4, 7, 4, 9, 5, 6, 5, 7, 5, 8, 5, 9, 5, 10,
];

#[rustfmt::skip]
const DATA_HEAWOOD: &[u32] = &[
    14, 21, 0,
    0, 1, 0, 5, 0, 13, 1, 2, 1, 10, 2, 3, 2, 7, 3, 4, 3, 12, 4, 5, 4, 9, 5, 6, 6,
    7, 6, 11, 7, 8, 8, 9, 8, 13, 9, 10, 10, 11, 11, 12, 12, 13,
];

#[rustfmt::skip]
const DATA_HERSCHEL: &[u32] = &[
    11, 18, 0,
    0, 2, 0, 3, 0, 4, 0, 5, 1, 2, 1, 3, 1, 6, 1, 7, 2, 10, 3, 9, 4, 8, 4, 9, 5, 8,
    5, 10, 6, 8, 6, 9, 7, 8, 7, 10,
];

#[rustfmt::skip]
const DATA_HOUSE: &[u32] = &[
    5, 6, 0,
    0, 1, 0, 2, 1, 3, 2, 3, 2, 4, 3, 4,
];

#[rustfmt::skip]
const DATA_HOUSEX: &[u32] = &[
    5, 8, 0,
    0, 1, 0, 2, 0, 3, 1, 2, 1, 3, 2, 3, 2, 4, 3, 4,
];

#[rustfmt::skip]
const DATA_ICOSAHEDRON: &[u32] = &[
    12, 30, 0,
    0, 1, 0, 2, 0, 3, 0, 4, 0, 8, 1, 2, 1, 6, 1, 7, 1, 8, 2, 4, 2, 5, 2, 6, 3, 4,
    3, 8, 3, 9, 3, 11, 4, 5, 4, 11, 5, 6, 5, 10, 5, 11, 6, 7, 6, 10, 7, 8, 7, 9, 7,
    10, 8, 9, 9, 10, 9, 11, 10, 11,
];

#[rustfmt::skip]
const DATA_KRACKHARDT_KITE: &[u32] = &[
    10, 18, 0,
    0, 1, 0, 2, 0, 3, 0, 5, 1, 3, 1, 4, 1, 6, 2, 3, 2, 5, 3, 4, 3, 5, 3, 6, 4, 6, 5, 6, 5, 7, 6, 7, 7, 8, 8, 9,
];

#[rustfmt::skip]
const DATA_LEVI: &[u32] = &[
    30, 45, 0,
    0, 1, 0, 7, 0, 29, 1, 2, 1, 24, 2, 3, 2, 11, 3, 4, 3, 16, 4, 5, 4, 21, 5, 6, 5,
    26, 6, 7, 6, 13, 7, 8, 8, 9, 8, 17, 9, 10, 9, 22, 10, 11, 10, 27, 11, 12, 12,
    13, 12, 19, 13, 14, 14, 15, 14, 23, 15, 16, 15, 28, 16, 17, 17, 18, 18, 19, 18,
    25, 19, 20, 20, 21, 20, 29, 21, 22, 22, 23, 23, 24, 24, 25, 25, 26, 26, 27, 27,
    28, 28, 29,
];

#[rustfmt::skip]
const DATA_MCGEE: &[u32] = &[
    24, 36, 0,
    0, 1, 0, 7, 0, 23, 1, 2, 1, 18, 2, 3, 2, 14, 3, 4, 3, 10, 4, 5, 4, 21, 5, 6, 5,
    17, 6, 7, 6, 13, 7, 8, 8, 9, 8, 20, 9, 10, 9, 16, 10, 11, 11, 12, 11, 23, 12,
    13, 12, 19, 13, 14, 14, 15, 15, 16, 15, 22, 16, 17, 17, 18, 18, 19, 19, 20, 20,
    21, 21, 22, 22, 23,
];

#[rustfmt::skip]
const DATA_MEREDITH: &[u32] = &[
    70, 140, 0,
    0, 4, 0, 5, 0, 6, 1, 4, 1, 5, 1, 6, 2, 4, 2, 5, 2, 6, 3, 4, 3, 5, 3, 6, 7, 11,
    7, 12, 7, 13, 8, 11, 8, 12, 8, 13, 9, 11, 9, 12, 9, 13, 10, 11, 10, 12, 10, 13,
    14, 18, 14, 19, 14, 20, 15, 18, 15, 19, 15, 20, 16, 18, 16, 19, 16, 20, 17, 18,
    17, 19, 17, 20, 21, 25, 21, 26, 21, 27, 22, 25, 22, 26, 22, 27, 23, 25, 23, 26,
    23, 27, 24, 25, 24, 26, 24, 27, 28, 32, 28, 33, 28, 34, 29, 32, 29, 33, 29, 34,
    30, 32, 30, 33, 30, 34, 31, 32, 31, 33, 31, 34, 35, 39, 35, 40, 35, 41, 36, 39,
    36, 40, 36, 41, 37, 39, 37, 40, 37, 41, 38, 39, 38, 40, 38, 41, 42, 46, 42, 47,
    42, 48, 43, 46, 43, 47, 43, 48, 44, 46, 44, 47, 44, 48, 45, 46, 45, 47, 45, 48,
    49, 53, 49, 54, 49, 55, 50, 53, 50, 54, 50, 55, 51, 53, 51, 54, 51, 55, 52, 53,
    52, 54, 52, 55, 56, 60, 56, 61, 56, 62, 57, 60, 57, 61, 57, 62, 58, 60, 58, 61,
    58, 62, 59, 60, 59, 61, 59, 62, 63, 67, 63, 68, 63, 69, 64, 67, 64, 68, 64, 69,
    65, 67, 65, 68, 65, 69, 66, 67, 66, 68, 66, 69, 2, 50, 1, 51, 9, 57, 8, 58, 16,
    64, 15, 65, 23, 36, 22, 37, 30, 43, 29, 44, 3, 21, 7, 24, 14, 31, 0, 17, 10,
    28, 38, 42, 35, 66, 59, 63, 52, 56, 45, 49,
];

#[rustfmt::skip]
const DATA_NOPERFECTMATCHING: &[u32] = &[
    16, 27, 0,
    0, 1, 0, 2, 0, 3, 1, 2, 1, 3, 2, 3, 2, 4, 3, 4, 4, 5, 5, 6, 5, 7, 6, 12, 6, 13,
    7, 8, 7, 9, 8, 9, 8, 10, 8, 11, 9, 10, 9, 11, 10, 11, 12, 13, 12, 14, 12, 15,
    13, 14, 13, 15, 14, 15,
];

#[rustfmt::skip]
const DATA_NONLINE: &[u32] = &[
    50, 72, 0,
    0, 1, 0, 2, 0, 3, 4, 6, 4, 7, 5, 6, 5, 7, 6, 7, 7, 8, 9, 11, 9, 12, 9, 13, 10,
    11, 10, 12, 10, 13, 11, 12, 11, 13, 12, 13, 14, 15, 15, 16, 15, 17, 16, 17, 16,
    18, 17, 18, 18, 19, 20, 21, 20, 22, 20, 23, 21, 22, 21, 23, 21, 24, 22, 23, 22,
    24, 24, 25, 26, 27, 26, 28, 26, 29, 27, 28, 27, 29, 27, 30, 27, 31, 28, 29, 28,
    30, 28, 31, 30, 31, 32, 34, 32, 35, 32, 36, 33, 34, 33, 35, 33, 37, 34, 35, 36,
    37, 38, 39, 38, 40, 38, 43, 39, 40, 39, 41, 39, 42, 39, 43, 40, 41, 41, 42, 42,
    43, 44, 45, 44, 46, 45, 46, 45, 47, 46, 47, 46, 48, 47, 48, 47, 49, 48, 49,
];

#[rustfmt::skip]
const DATA_OCTAHEDRON: &[u32] = &[
    6, 12, 0,
    0, 1, 0, 2, 1, 2, 3, 4, 3, 5, 4, 5, 0, 3, 0, 5, 1, 3, 1, 4, 2, 4, 2, 5,
];

#[rustfmt::skip]
const DATA_PETERSEN: &[u32] = &[
    10, 15, 0,
    0, 1, 0, 4, 0, 5, 1, 2, 1, 6, 2, 3, 2, 7, 3, 4, 3, 8, 4, 9, 5, 7, 5, 8, 6, 8, 6, 9, 7, 9,
];

#[rustfmt::skip]
const DATA_ROBERTSON: &[u32] = &[
    19, 38, 0,
    0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12,
    12, 13, 13, 14, 14, 15, 15, 16, 16, 17, 17, 18, 0, 18, 0, 4, 4, 9, 9, 13, 13,
    17, 2, 17, 2, 6, 6, 10, 10, 15, 0, 15, 1, 8, 8, 16, 5, 16, 5, 12, 1, 12, 7, 18,
    7, 14, 3, 14, 3, 11, 11, 18,
];

#[rustfmt::skip]
const DATA_SMALLESTCYCLICGROUP: &[u32] = &[
    9, 15, 0,
    0, 1, 0, 2, 0, 3, 0, 4, 0, 5, 1, 2, 1, 3, 1, 7, 1, 8, 2, 5, 2, 6, 2, 7, 3, 8,
    4, 5, 6, 7,
];

#[rustfmt::skip]
const DATA_TETRAHEDRON: &[u32] = &[
    4, 6, 0,
    0, 3, 1, 3, 2, 3, 0, 1, 1, 2, 0, 2,
];

#[rustfmt::skip]
const DATA_THOMASSEN: &[u32] = &[
    34, 52, 0,
    0, 2, 0, 3, 1, 3, 1, 4, 2, 4, 5, 7, 5, 8, 6, 8, 6, 9, 7, 9, 10, 12, 10, 13, 11,
    13, 11, 14, 12, 14, 15, 17, 15, 18, 16, 18, 16, 19, 17, 19, 9, 19, 4, 14, 24,
    25, 25, 26, 20, 26, 20, 21, 21, 22, 22, 23, 23, 27, 27, 28, 28, 29, 29, 30, 30,
    31, 31, 32, 32, 33, 24, 33, 5, 24, 6, 25, 7, 26, 8, 20, 0, 20, 1, 21, 2, 22, 3,
    23, 10, 27, 11, 28, 12, 29, 13, 30, 15, 30, 16, 31, 17, 32, 18, 33,
];

#[rustfmt::skip]
const DATA_TUTTE: &[u32] = &[
    46, 69, 0,
    0, 10, 0, 11, 0, 12, 1, 2, 1, 7, 1, 19, 2, 3, 2, 41, 3, 4, 3, 27, 4, 5, 4, 33,
    5, 6, 5, 45, 6, 9, 6, 29, 7, 8, 7, 21, 8, 9, 8, 22, 9, 24, 10, 13, 10, 14, 11,
    26, 11, 28, 12, 30, 12, 31, 13, 15, 13, 21, 14, 15, 14, 18, 15, 16, 16, 17, 16,
    20, 17, 18, 17, 23, 18, 24, 19, 25, 19, 40, 20, 21, 20, 22, 22, 23, 23, 24, 25,
    26, 25, 38, 26, 34, 27, 28, 27, 39, 28, 34, 29, 30, 29, 44, 30, 35, 31, 32, 31,
    35, 32, 33, 32, 42, 33, 43, 34, 36, 35, 37, 36, 38, 36, 39, 37, 42, 37, 44, 38,
    40, 39, 41, 40, 41, 42, 43, 43, 45, 44, 45,
];

#[rustfmt::skip]
const DATA_UNIQUELY3COLORABLE: &[u32] = &[
    12, 22, 0,
    0, 1, 0, 3, 0, 6, 0, 8, 1, 4, 1, 7, 1, 9, 2, 3, 2, 6, 2, 7, 2, 9, 2, 11, 3, 4,
    3, 10, 4, 5, 4, 11, 5, 6, 5, 7, 5, 8, 5, 10, 8, 11, 9, 10,
];

#[rustfmt::skip]
const DATA_WALTHER: &[u32] = &[
    25, 31, 0,
    0, 1, 1, 2, 1, 8, 2, 3, 2, 13, 3, 4, 3, 16, 4, 5, 5, 6, 5, 19, 6, 7, 6, 20, 7,
    21, 8, 9, 8, 13, 9, 10, 9, 22, 10, 11, 10, 20, 11, 12, 13, 14, 14, 15, 14, 23,
    15, 16, 15, 17, 17, 18, 18, 19, 18, 24, 20, 24, 22, 23, 23, 24,
];

#[rustfmt::skip]
const DATA_ZACHARY: &[u32] = &[
    34, 78, 0,
    0, 1, 0, 2, 0, 3, 0, 4, 0, 5, 0, 6, 0, 7, 0, 8,
    0, 10, 0, 11, 0, 12, 0, 13, 0, 17, 0, 19, 0, 21, 0, 31,
    1, 2, 1, 3, 1, 7, 1, 13, 1, 17, 1, 19, 1, 21, 1, 30,
    2, 3, 2, 7, 2, 27, 2, 28, 2, 32, 2, 9, 2, 8, 2, 13,
    3, 7, 3, 12, 3, 13, 4, 6, 4, 10, 5, 6, 5, 10, 5, 16,
    6, 16, 8, 30, 8, 32, 8, 33, 9, 33, 13, 33, 14, 32, 14, 33,
    15, 32, 15, 33, 18, 32, 18, 33, 19, 33, 20, 32, 20, 33,
    22, 32, 22, 33, 23, 25, 23, 27, 23, 32, 23, 33, 23, 29,
    24, 25, 24, 27, 24, 31, 25, 31, 26, 29, 26, 33, 27, 33,
    28, 31, 28, 33, 29, 32, 29, 33, 30, 32, 30, 33, 31, 32, 31, 33,
    32, 33,
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn canonical_edge_set(g: &Graph) -> BTreeSet<(u32, u32)> {
        let mut s = BTreeSet::new();
        for v in 0..g.vcount() {
            for &u in &g.neighbors(v).expect("neighbors") {
                if v <= u {
                    s.insert((v, u));
                } else {
                    s.insert((u, v));
                }
            }
        }
        s
    }

    #[test]
    fn unknown_name_errors() {
        assert!(famous("nonexistent").is_err());
        assert!(famous("").is_err());
    }

    #[test]
    fn names_are_case_insensitive() {
        let p1 = famous("PETERSEN").expect("upper");
        let p2 = famous("Petersen").expect("title");
        let p3 = famous("petersen").expect("lower");
        let p4 = famous("PeTeRsEn").expect("mixed");
        for p in [&p2, &p3, &p4] {
            assert_eq!(p1.vcount(), p.vcount());
            assert_eq!(p1.ecount(), p.ecount());
            assert_eq!(canonical_edge_set(&p1), canonical_edge_set(p));
        }
    }

    #[test]
    fn aliases_match_primary() {
        let pairs = [
            ("dodecahedral", "dodecahedron"),
            ("grotzsch", "groetzsch"),
            ("icosahedral", "icosahedron"),
            ("octahedral", "octahedron"),
            ("tetrahedral", "tetrahedron"),
        ];
        for (a, b) in pairs {
            let ga = famous(a).expect(a);
            let gb = famous(b).expect(b);
            assert_eq!(ga.vcount(), gb.vcount(), "{a} vs {b} vcount");
            assert_eq!(ga.ecount(), gb.ecount(), "{a} vs {b} ecount");
            assert_eq!(canonical_edge_set(&ga), canonical_edge_set(&gb));
        }
    }

    #[test]
    fn every_name_in_table_builds() {
        for name in famous_names() {
            let g = famous(name).unwrap_or_else(|_| panic!("famous({name}) failed"));
            assert!(g.vcount() > 0, "{name} should have at least one vertex");
            assert!(!g.is_directed(), "{name} should be undirected");
        }
    }

    #[test]
    fn vertex_and_edge_counts_match_data_table() {
        // Spot-checks against the famous.c data header rows.
        let cases = [
            ("bull", 5, 5),
            ("chvatal", 12, 24),
            ("coxeter", 28, 42),
            ("cubical", 8, 12),
            ("diamond", 4, 5),
            ("dodecahedron", 20, 30),
            ("folkman", 20, 40),
            ("franklin", 12, 18),
            ("frucht", 12, 18),
            ("grotzsch", 11, 20),
            ("heawood", 14, 21),
            ("herschel", 11, 18),
            ("house", 5, 6),
            ("housex", 5, 8),
            ("icosahedron", 12, 30),
            ("krackhardt_kite", 10, 18),
            ("levi", 30, 45),
            ("mcgee", 24, 36),
            ("meredith", 70, 140),
            ("noperfectmatching", 16, 27),
            ("nonline", 50, 72),
            ("octahedron", 6, 12),
            ("petersen", 10, 15),
            ("robertson", 19, 38),
            ("smallestcyclicgroup", 9, 15),
            ("tetrahedron", 4, 6),
            ("thomassen", 34, 52),
            ("tutte", 46, 69),
            ("uniquely3colorable", 12, 22),
            ("walther", 25, 31),
            ("zachary", 34, 78),
        ];
        for (name, v, e) in cases {
            let g = famous(name).expect(name);
            assert_eq!(g.vcount(), v, "{name} vcount");
            assert_eq!(g.ecount(), e, "{name} ecount");
        }
    }

    #[test]
    fn bull_has_expected_edges() {
        let bull = famous("bull").expect("bull");
        let want: BTreeSet<(u32, u32)> = [(0, 1), (0, 2), (1, 2), (1, 3), (2, 4)]
            .into_iter()
            .collect();
        assert_eq!(canonical_edge_set(&bull), want);
    }

    #[test]
    fn tetrahedron_is_k4() {
        // K_4 — every pair of distinct vertices connected exactly once.
        let g = famous("tetrahedron").expect("t");
        let want: BTreeSet<(u32, u32)> = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]
            .into_iter()
            .collect();
        assert_eq!(canonical_edge_set(&g), want);
    }

    #[test]
    fn petersen_is_3_regular() {
        let g = famous("petersen").expect("p");
        for v in 0..g.vcount() {
            assert_eq!(g.neighbors(v).unwrap().len(), 3, "vertex {v} degree");
        }
    }

    #[test]
    fn no_self_loops_in_undirected_famous() {
        for name in famous_names() {
            let g = famous(name).unwrap();
            for v in 0..g.vcount() {
                for &u in &g.neighbors(v).unwrap() {
                    assert_ne!(v, u, "{name} has self-loop at {v}");
                }
            }
        }
    }

    #[test]
    fn famous_names_match_table_canonical_entries() {
        // famous_names() should list exactly the canonical names (no aliases).
        let names: BTreeSet<&str> = famous_names().iter().copied().collect();
        assert_eq!(names.len(), famous_names().len(), "no duplicate names");
        // Every name must be accepted by famous() itself.
        for n in &names {
            assert!(
                famous(n).is_ok(),
                "famous_names() entry '{n}' is not accepted"
            );
        }
    }
}
