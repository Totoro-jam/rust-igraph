//! Cohesive blocking (ALGO-CN-032).
//!
//! Counterpart of `igraph_cohesive_blocks()` from
//! `references/igraph/src/connectivity/cohesive_blocks.c`.
//!
//! Cohesive blocking (Moody & White, 2003) finds the hierarchical
//! structure of maximally k-cohesive vertex subsets of a graph: starting
//! from the whole graph, it recursively removes minimum-size vertex
//! separators and keeps the resulting subgraphs whose connectivity is
//! strictly higher than that of their parent.

use std::collections::VecDeque;

use crate::algorithms::connectivity::separators::minimum_size_separators;
use crate::algorithms::constructors::create::create;
use crate::algorithms::flow::vertex_connectivity::vertex_connectivity;
use crate::algorithms::operators::induced_subgraph::induced_subgraph;
use crate::algorithms::properties::degree::{DegreeMode, max_degree};
use crate::algorithms::properties::is_simple::is_simple;
use crate::core::{Graph, IgraphError, IgraphResult, VertexId};

/// Result of [`cohesive_blocks`].
///
/// All four members are index-aligned: entry `i` of [`cohesion`](Self::cohesion)
/// and [`parent`](Self::parent) describes block `i` of [`blocks`](Self::blocks),
/// and vertex `i` of [`block_tree`](Self::block_tree) is the same block.
#[derive(Debug, Clone)]
pub struct CohesiveBlocks {
    /// The cohesive blocks; each is the sorted list of original vertex IDs
    /// that make up the block. Block `0` is always the whole graph.
    pub blocks: Vec<Vec<VertexId>>,
    /// The vertex connectivity (structural cohesion) of each block.
    pub cohesion: Vec<i64>,
    /// The block-hierarchy parent of each block (an index into
    /// [`blocks`](Self::blocks)), or `-1` for the root block.
    pub parent: Vec<i64>,
    /// The block hierarchy as a directed graph: one vertex per block, an
    /// arc from each block's parent to the block.
    pub block_tree: Graph,
}

/// Connected components of `graph` after deleting the `excluded` vertices.
///
/// Mirrors `igraph_i_cb_components`: excluded vertices are not traversed,
/// but every excluded vertex that borders a component is appended to that
/// component (so a separator vertex can appear in several components).
/// Each returned component lists its non-excluded vertices in BFS order
/// followed by the bordering excluded vertices in discovery order.
fn cb_components(graph: &Graph, excluded: &[bool]) -> IgraphResult<Vec<Vec<VertexId>>> {
    let n = graph.vcount() as usize;
    let mut compid = vec![0usize; n];
    let mut comps: Vec<Vec<VertexId>> = Vec::new();
    let mut cno = 0usize;
    let mut q: VecDeque<VertexId> = VecDeque::new();

    for i in 0..n {
        if compid[i] != 0 || excluded[i] {
            continue;
        }
        cno += 1;
        let mut comp: Vec<VertexId> = Vec::new();
        let start = u32::try_from(i)
            .map_err(|_| IgraphError::InvalidArgument("cohesive_blocks: vertex overflow".into()))?;
        q.push_back(start);
        comp.push(start);
        compid[i] = cno;

        while let Some(node) = q.pop_front() {
            for v in graph.neighbors(node)? {
                let vi = v as usize;
                if excluded[vi] {
                    if compid[vi] != cno {
                        compid[vi] = cno;
                        comp.push(v);
                    }
                } else if compid[vi] == 0 {
                    compid[vi] = cno;
                    comp.push(v);
                    q.push_back(v);
                }
            }
        }
        comps.push(comp);
    }

    Ok(comps)
}

/// Subset test for two ascending-sorted vertex lists: is `needle ⊆ haystack`?
///
/// Mirrors `igraph_i_cb_isin`; both slices must be sorted ascending.
fn cb_isin(needle: &[VertexId], haystack: &[VertexId]) -> bool {
    if haystack.len() < needle.len() {
        return false;
    }
    let mut np = 0;
    let mut hp = 0;
    while np < needle.len() && hp < haystack.len() {
        match needle[np].cmp(&haystack[hp]) {
            std::cmp::Ordering::Equal => {
                np += 1;
                hp += 1;
            }
            std::cmp::Ordering::Less => return false,
            std::cmp::Ordering::Greater => hp += 1,
        }
    }
    np == needle.len()
}

/// Convert a known-non-negative `i64` queue index into a `usize`.
fn qidx(p: i64) -> IgraphResult<usize> {
    usize::try_from(p)
        .map_err(|_| IgraphError::InvalidArgument("cohesive_blocks: negative queue index".into()))
}

/// Identify the hierarchical cohesive block structure of a graph.
///
/// Cohesive blocking (J. Moody and D. R. White, "Structural cohesion and
/// embeddedness: A hierarchical concept of social groups", American
/// Sociological Review 68(1):103–127, 2003) determines nested subsets of
/// vertices ordered by their structural cohesion (vertex connectivity).
/// The whole graph is the root block; each block's maximally more-cohesive
/// subsets are found recursively by removing minimum-size separators.
///
/// The returned [`CohesiveBlocks`] reports, for every block, its vertex
/// set (sorted ascending), its cohesion, its parent in the hierarchy, and
/// the hierarchy itself as a directed tree. Block `0` is always the whole
/// graph with `parent = -1`.
///
/// For undirected, simple graphs only.
///
/// # Errors
///
/// - [`IgraphError::InvalidArgument`] if the graph is directed.
/// - [`IgraphError::InvalidArgument`] if the graph is not simple (has
///   self-loops or multiple edges).
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, cohesive_blocks};
///
/// // A 4-clique 0-1-2-3 with a pendant path 3-4-5: the clique is a more
/// // cohesive block (connectivity 3) inside the whole graph.
/// let mut g = Graph::with_vertices(6);
/// for (a, b) in [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3), (3, 4), (4, 5)] {
///     g.add_edge(a, b).unwrap();
/// }
/// let cb = cohesive_blocks(&g).unwrap();
/// assert_eq!(cb.blocks[0], (0..6).collect::<Vec<_>>());
/// assert_eq!(cb.parent[0], -1);
/// // The 4-clique appears as a block with cohesion 3.
/// assert!(cb.blocks.iter().zip(&cb.cohesion).any(|(b, &c)| b == &[0, 1, 2, 3] && c == 3));
/// ```
#[allow(clippy::too_many_lines)]
pub fn cohesive_blocks(graph: &Graph) -> IgraphResult<CohesiveBlocks> {
    if graph.is_directed() {
        return Err(IgraphError::InvalidArgument(
            "cohesive_blocks: only works on undirected graphs".into(),
        ));
    }
    if !is_simple(graph)? {
        return Err(IgraphError::InvalidArgument(
            "cohesive_blocks: only works on simple graphs".into(),
        ));
    }

    // The work queue of subgraphs, with parallel bookkeeping arrays.
    // `q_graph[i]` is a subgraph; `q_mapping[i]` maps its vertex IDs to the
    // parent's vertex IDs (later rewritten to original IDs); `q_parent[i]`
    // is the index of its parent in the queue; `q_cohesion[i]` its vertex
    // connectivity; `q_check[i]` flags blocks born from a separator split
    // (these need the containment dedup pass).
    let mut q_graph: Vec<Option<Graph>> = Vec::new();
    let mut q_mapping: Vec<Vec<VertexId>> = Vec::new();
    let mut q_parent: Vec<i64> = Vec::new();
    let mut q_cohesion: Vec<i64> = Vec::new();
    let mut q_check: Vec<bool> = Vec::new();

    q_graph.push(Some(graph.clone()));
    q_mapping.push(Vec::new());
    q_parent.push(-1);
    q_cohesion.push(vertex_connectivity(graph, true)?);
    q_check.push(false);

    let mut qptr = 0usize;
    while qptr < q_graph.len() {
        let mygraph = q_graph[qptr]
            .take()
            .ok_or_else(|| IgraphError::InvalidArgument("cohesive_blocks: queue slot".into()))?;
        let mycheck = q_check[qptr];
        let my_cohesion = q_cohesion[qptr];
        let mynodes = mygraph.vcount() as usize;

        // Minimum-size separators of the current block.
        let separators = minimum_size_separators(&mygraph)?;

        // Mark all separator vertices.
        let mut marked = vec![false; mynodes];
        let mut nsepv = 0usize;
        for sep in &separators {
            for &vv in sep {
                let vi = vv as usize;
                if !marked[vi] {
                    nsepv += 1;
                    marked[vi] = true;
                }
            }
        }

        // Components after removing the separators (bordering separators
        // are kept attached to each touching component).
        let mut components = cb_components(&mygraph, &marked)?;

        // Add the separator vertices themselves as one more component, but
        // only if some vertex is outside every separator.
        let addedsep = nsepv != mynodes;
        if addedsep {
            let sep_comp: Vec<VertexId> = (0..mynodes)
                .filter(|&i| marked[i])
                .map(u32::try_from)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| {
                    IgraphError::InvalidArgument("cohesive_blocks: vertex overflow".into())
                })?;
            components.push(sep_comp);
        }

        for compvertices in components {
            let sub = induced_subgraph(&mygraph, &compvertices)?;
            let maxdeg = i64::from(max_degree(&sub.graph, DegreeMode::All)?);
            if maxdeg > my_cohesion {
                let newconn = vertex_connectivity(&sub.graph, true)?;
                q_graph.push(Some(sub.graph));
                q_mapping.push(sub.invmap);
                q_cohesion.push(newconn);
                q_parent.push(i64::try_from(qptr).map_err(|_| {
                    IgraphError::InvalidArgument("cohesive_blocks: queue overflow".into())
                })?);
                q_check.push(mycheck || addedsep);
            }
        }

        qptr += 1;
    }

    let noblocks_full = qptr;
    let mut removed = vec![false; noblocks_full];
    let mut rewritemap = vec![0i64; noblocks_full];
    let mut badblocks = 0usize;

    // Drop a block whose nearest surviving ancestor is at least as cohesive.
    for i in 1..noblocks_full {
        let mut p = qidx(q_parent[i])?;
        while removed[p] {
            p = qidx(q_parent[p])?;
        }
        if q_cohesion[p] >= q_cohesion[i] {
            removed[i] = true;
            badblocks += 1;
        }
    }

    // Rewrite each mapping up one level until every mapping is in terms of
    // the original graph's vertex IDs. Parents are processed before
    // children (parent index < child index), so the parent's mapping is
    // already in original IDs when we reach the child.
    for i in 1..noblocks_full {
        let p = qidx(q_parent[i])?;
        if p == 0 {
            continue;
        }
        let pmapping = q_mapping[p].clone();
        for v in &mut q_mapping[i] {
            *v = pmapping[*v as usize];
        }
    }

    // Separator-as-block components can be subsets of other blocks; drop a
    // checked block that is contained in another checked block of at least
    // equal cohesion.
    for i in 1..noblocks_full {
        if !q_check[i] || removed[i] {
            continue;
        }
        let ic = q_cohesion[i];
        for j in 1..noblocks_full {
            if j == i || !q_check[j] || removed[j] {
                continue;
            }
            if q_cohesion[j] >= ic && cb_isin(&q_mapping[i], &q_mapping[j]) {
                badblocks += 1;
                removed[i] = true;
                break;
            }
        }
    }

    let noblocks = noblocks_full - badblocks;

    let mut blocks: Vec<Vec<VertexId>> = vec![Vec::new(); noblocks];
    let mut cohesion = vec![0i64; noblocks];
    let mut parent = vec![0i64; noblocks];

    let mut resptr = 0usize;
    for i in 0..noblocks_full {
        if removed[i] {
            continue;
        }
        rewritemap[i] = i64::try_from(resptr)
            .map_err(|_| IgraphError::InvalidArgument("cohesive_blocks: block overflow".into()))?;
        cohesion[resptr] = q_cohesion[i];

        let mut p = q_parent[i];
        while p >= 0 && removed[qidx(p)?] {
            p = q_parent[qidx(p)?];
        }
        if p >= 0 {
            p = rewritemap[qidx(p)?];
        }
        q_parent[i] = p;
        parent[resptr] = p;

        blocks[resptr] = std::mem::take(&mut q_mapping[i]);
        resptr += 1;
    }

    // Block 0 is the whole graph.
    blocks[0] = (0..graph.vcount()).collect();

    // Build the directed block tree (parent -> child).
    let mut edges: Vec<(VertexId, VertexId)> = Vec::with_capacity(noblocks.saturating_sub(1));
    for i in 1..noblocks_full {
        if removed[i] {
            continue;
        }
        let p = u32::try_from(q_parent[i])
            .map_err(|_| IgraphError::InvalidArgument("cohesive_blocks: tree parent".into()))?;
        let c = u32::try_from(rewritemap[i])
            .map_err(|_| IgraphError::InvalidArgument("cohesive_blocks: tree child".into()))?;
        edges.push((p, c));
    }
    let block_tree = create(&edges, u32::try_from(noblocks).unwrap_or(0), true)?;

    Ok(CohesiveBlocks {
        blocks,
        cohesion,
        parent,
        block_tree,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ug(n: u32, edges: &[(u32, u32)]) -> Graph {
        let mut g = Graph::with_vertices(n);
        for &(a, b) in edges {
            g.add_edge(a, b).expect("edge in range");
        }
        g
    }

    /// Canonicalise the output as a set of (sorted block, cohesion) pairs so
    /// comparisons are independent of block enumeration order.
    fn canon(cb: &CohesiveBlocks) -> Vec<(Vec<VertexId>, i64)> {
        let mut v: Vec<(Vec<VertexId>, i64)> = cb
            .blocks
            .iter()
            .zip(&cb.cohesion)
            .map(|(b, &c)| {
                let mut bb = b.clone();
                bb.sort_unstable();
                (bb, c)
            })
            .collect();
        v.sort();
        v
    }

    #[test]
    fn rejects_directed() {
        let mut g = Graph::new(3, true).expect("graph");
        g.add_edge(0, 1).expect("edge");
        assert!(cohesive_blocks(&g).is_err());
    }

    #[test]
    fn rejects_non_simple() {
        let mut g = Graph::with_vertices(3);
        g.add_edge(0, 1).expect("edge");
        g.add_edge(0, 1).expect("edge"); // multi-edge
        assert!(cohesive_blocks(&g).is_err());
    }

    #[test]
    fn single_block_clique() {
        // K4 has a single cohesive block: the whole graph, cohesion 3.
        let g = ug(4, &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]);
        let cb = cohesive_blocks(&g).expect("cohesive_blocks");
        assert_eq!(cb.blocks.len(), 1);
        assert_eq!(cb.blocks[0], vec![0, 1, 2, 3]);
        assert_eq!(cb.cohesion, vec![3]);
        assert_eq!(cb.parent, vec![-1]);
    }

    #[test]
    fn moody_white() {
        // The graph from the Moody-White paper (igraph C test fixture).
        let g = ug(
            23,
            &[
                (0, 1),
                (0, 2),
                (0, 3),
                (0, 4),
                (0, 5),
                (1, 2),
                (1, 3),
                (1, 4),
                (1, 6),
                (2, 3),
                (2, 5),
                (2, 6),
                (3, 4),
                (3, 5),
                (3, 6),
                (4, 5),
                (4, 6),
                (4, 20),
                (5, 6),
                (6, 7),
                (6, 10),
                (6, 13),
                (6, 18),
                (7, 8),
                (7, 10),
                (7, 13),
                (8, 9),
                (9, 11),
                (9, 12),
                (10, 11),
                (10, 13),
                (11, 15),
                (12, 15),
                (13, 14),
                (14, 15),
                (16, 17),
                (16, 18),
                (16, 19),
                (17, 19),
                (17, 20),
                (18, 19),
                (18, 21),
                (18, 22),
                (19, 20),
                (20, 21),
                (20, 22),
                (21, 22),
            ],
        );
        let cb = cohesive_blocks(&g).expect("cohesive_blocks");
        let got = canon(&cb);
        let mut expected = vec![
            ((0..23).collect::<Vec<_>>(), 1),
            (vec![0, 1, 2, 3, 4, 5, 6, 16, 17, 18, 19, 20, 21, 22], 2),
            (vec![6, 7, 8, 9, 10, 11, 12, 13, 14, 15], 2),
            (vec![0, 1, 2, 3, 4, 5, 6], 5),
            (vec![6, 7, 10, 13], 3),
        ];
        expected.sort();
        assert_eq!(got, expected);
    }

    #[test]
    fn tricky_separators_form_a_block() {
        let g = ug(
            8,
            &[
                (0, 1),
                (0, 4),
                (0, 5),
                (1, 2),
                (1, 4),
                (1, 5),
                (1, 6),
                (2, 3),
                (2, 5),
                (2, 6),
                (2, 7),
                (3, 6),
                (3, 7),
                (4, 5),
                (5, 6),
                (6, 7),
            ],
        );
        let cb = cohesive_blocks(&g).expect("cohesive_blocks");
        let got = canon(&cb);
        let mut expected = vec![
            ((0..8).collect::<Vec<_>>(), 2),
            (vec![0, 1, 4, 5], 3),
            (vec![2, 3, 6, 7], 3),
            (vec![1, 2, 5, 6], 3),
        ];
        expected.sort();
        assert_eq!(got, expected);
    }

    #[test]
    fn science_camp() {
        let g = ug(
            18,
            &[
                (0, 1),
                (0, 2),
                (0, 3),
                (1, 2),
                (1, 3),
                (1, 16),
                (1, 17),
                (2, 3),
                (3, 17),
                (4, 5),
                (4, 6),
                (4, 7),
                (4, 8),
                (5, 6),
                (5, 7),
                (6, 7),
                (6, 8),
                (7, 8),
                (7, 16),
                (8, 9),
                (8, 10),
                (9, 11),
                (9, 12),
                (9, 13),
                (9, 14),
                (10, 11),
                (10, 12),
                (10, 13),
                (11, 14),
                (12, 13),
                (12, 14),
                (12, 15),
                (15, 16),
                (15, 17),
                (16, 17),
            ],
        );
        let cb = cohesive_blocks(&g).expect("cohesive_blocks");
        let got = canon(&cb);
        let mut expected = vec![
            ((0..18).collect::<Vec<_>>(), 2),
            (vec![0, 1, 2, 3], 3),
            (vec![4, 5, 6, 7, 8], 3),
            (vec![9, 10, 11, 12, 13, 14], 3),
        ];
        expected.sort();
        assert_eq!(got, expected);
    }

    #[test]
    fn root_is_whole_graph_and_tree_consistent() {
        let g = ug(
            6,
            &[
                (0, 1),
                (0, 2),
                (0, 3),
                (1, 2),
                (1, 3),
                (2, 3),
                (3, 4),
                (4, 5),
            ],
        );
        let cb = cohesive_blocks(&g).expect("cohesive_blocks");
        assert_eq!(cb.blocks[0], (0..6).collect::<Vec<_>>());
        assert_eq!(cb.parent[0], -1);
        // block_tree vertex count equals number of blocks; one arc per non-root.
        assert_eq!(cb.block_tree.vcount() as usize, cb.blocks.len());
        let non_root = cb.parent.iter().filter(|&&p| p >= 0).count();
        assert_eq!(cb.block_tree.ecount() as usize, non_root);
    }
}
