//! MCMC-based HRG fitting engine.
//!
//! Implements the Clauset-Moore-Newman (2008) algorithm for fitting
//! hierarchical random graph models via Markov Chain Monte Carlo over
//! the space of dendrograms. The engine supports:
//! - [`hrg_fit`]: MCMC search for the maximum-likelihood dendrogram
//! - [`hrg_consensus`]: sample split frequencies from MCMC
//! - [`hrg_predict`]: predict missing edges based on MCMC ensemble

#![allow(
    clippy::doc_markdown,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_lossless,
    clippy::needless_range_loop,
    clippy::many_single_char_names
)]

use crate::core::rng::SplitMix64;
use crate::core::{Graph, IgraphError, IgraphResult};

use super::HrgTree;

// ── Internal dendrogram representation ──────────────────────────────────

const LEFT: u8 = 0;
const RIGHT: u8 = 1;

/// A child pointer: either a leaf vertex or an internal node index.
#[derive(Debug, Clone, Copy)]
enum Child {
    Leaf(u32),
    Internal(usize),
}

/// An internal node in the dendrogram.
#[derive(Debug, Clone)]
struct DendroNode {
    left: Child,
    right: Child,
    parent: Option<usize>,
    /// Number of leaves in this subtree.
    n: u32,
    /// Number of edges spanning left and right subtrees.
    e: u32,
    /// Probability p = e / (nL * nR).
    p: f64,
    /// Log-likelihood contribution of this node.
    log_l: f64,
    /// Minimum leaf label in subtree (for order property).
    label: u32,
}

/// The full dendrogram structure for MCMC.
struct Dendro {
    nodes: Vec<DendroNode>,
    /// Parent of leaf v (index of the internal node).
    leaf_parent: Vec<usize>,
    /// Which side leaf v is on relative to its parent (LEFT or RIGHT).
    leaf_side: Vec<u8>,
    /// Adjacency list for the input graph (symmetric).
    adj: Vec<Vec<u32>>,
    /// Total log-likelihood of current dendrogram.
    total_log_l: f64,
    /// Number of leaf vertices.
    n: usize,
}

impl Dendro {
    /// Build a random initial dendrogram from a graph.
    #[allow(clippy::cast_possible_truncation)]
    fn from_graph(graph: &Graph, rng: &mut SplitMix64) -> IgraphResult<Self> {
        let n = graph.vcount() as usize;
        if n < 3 {
            return Err(IgraphError::InvalidArgument(
                "HRG fit requires at least 3 vertices".into(),
            ));
        }

        // Build symmetric adjacency list (ignoring self-loops + multi-edges)
        let adj = build_adjacency(graph)?;

        // Create n-1 internal nodes with random initial structure.
        // Strategy: random sequential insertion (like the C code).
        let num_internal = n - 1;
        let mut nodes: Vec<DendroNode> = Vec::with_capacity(num_internal);
        for _ in 0..num_internal {
            nodes.push(DendroNode {
                left: Child::Leaf(0),
                right: Child::Leaf(0),
                parent: None,
                n: 0,
                e: 0,
                p: 0.0,
                log_l: 0.0,
                label: 0,
            });
        }

        let mut leaf_parent = vec![0usize; n];
        let mut leaf_side = vec![LEFT; n];

        // Random permutation of leaves
        let mut perm: Vec<u32> = (0..n as u32).collect();
        for i in (1..n).rev() {
            let j = rng.gen_index(i + 1);
            perm.swap(i, j);
        }

        // Build a random binary tree by inserting leaves one at a time.
        // Start with root having first two leaves.
        nodes[0].left = Child::Leaf(perm[0]);
        nodes[0].right = Child::Leaf(perm[1]);
        leaf_parent[perm[0] as usize] = 0;
        leaf_side[perm[0] as usize] = LEFT;
        leaf_parent[perm[1] as usize] = 0;
        leaf_side[perm[1] as usize] = RIGHT;

        // For each remaining leaf, pick a random existing leaf and
        // replace it with a new internal node that has the old leaf
        // and new leaf as children.
        let mut active_internal = 1usize; // next internal node to allocate
        for leaf_idx in 2..n {
            let new_leaf = perm[leaf_idx];
            // Pick a random existing leaf to split
            let target_leaf = perm[rng.gen_index(leaf_idx)];
            let target_parent = leaf_parent[target_leaf as usize];
            let target_side = leaf_side[target_leaf as usize];

            let new_internal = active_internal;
            active_internal += 1;

            // New internal node gets target_leaf and new_leaf as children
            nodes[new_internal].left = Child::Leaf(target_leaf);
            nodes[new_internal].right = Child::Leaf(new_leaf);
            nodes[new_internal].parent = Some(target_parent);

            // Replace the target_leaf in its parent with the new internal node
            if target_side == LEFT {
                nodes[target_parent].left = Child::Internal(new_internal);
            } else {
                nodes[target_parent].right = Child::Internal(new_internal);
            }

            // Update leaf parents
            leaf_parent[target_leaf as usize] = new_internal;
            leaf_side[target_leaf as usize] = LEFT;
            leaf_parent[new_leaf as usize] = new_internal;
            leaf_side[new_leaf as usize] = RIGHT;
        }

        let mut dendro = Dendro {
            nodes,
            leaf_parent,
            leaf_side,
            adj,
            total_log_l: 0.0,
            n,
        };

        dendro.recompute_all();
        Ok(dendro)
    }

    /// Build dendro from an existing HRG tree + graph.
    #[allow(clippy::needless_range_loop, clippy::cast_possible_truncation)]
    fn from_hrg(graph: &Graph, hrg: &HrgTree) -> IgraphResult<Self> {
        let n = graph.vcount() as usize;
        if n < 3 {
            return Err(IgraphError::InvalidArgument(
                "HRG fit requires at least 3 vertices".into(),
            ));
        }
        if hrg.size() as usize != n {
            return Err(IgraphError::InvalidArgument(
                "HRG size does not match graph vertex count".into(),
            ));
        }

        let adj = build_adjacency(graph)?;
        let num_internal = n - 1;

        let mut nodes: Vec<DendroNode> = Vec::with_capacity(num_internal);
        let mut leaf_parent = vec![0usize; n];
        let mut leaf_side = vec![LEFT; n];

        for i in 0..num_internal {
            let lc = hrg.left[i];
            let rc = hrg.right[i];

            let left = if lc < 0 {
                #[allow(clippy::cast_sign_loss)]
                let idx = (-lc - 1) as usize;
                Child::Internal(idx)
            } else {
                #[allow(clippy::cast_sign_loss)]
                let idx = lc as u32;
                leaf_parent[idx as usize] = i;
                leaf_side[idx as usize] = LEFT;
                Child::Leaf(idx)
            };

            let right = if rc < 0 {
                #[allow(clippy::cast_sign_loss)]
                let idx = (-rc - 1) as usize;
                Child::Internal(idx)
            } else {
                #[allow(clippy::cast_sign_loss)]
                let idx = rc as u32;
                leaf_parent[idx as usize] = i;
                leaf_side[idx as usize] = RIGHT;
                Child::Leaf(idx)
            };

            nodes.push(DendroNode {
                left,
                right,
                parent: None,
                n: hrg.vertices[i] as u32,
                e: hrg.edges[i] as u32,
                p: hrg.prob[i],
                log_l: 0.0,
                label: 0,
            });
        }

        // Set parent pointers
        for i in 0..num_internal {
            match nodes[i].left {
                Child::Internal(c) => nodes[c].parent = Some(i),
                Child::Leaf(_) => {}
            }
            match nodes[i].right {
                Child::Internal(c) => nodes[c].parent = Some(i),
                Child::Leaf(_) => {}
            }
        }

        let mut dendro = Dendro {
            nodes,
            leaf_parent,
            leaf_side,
            adj,
            total_log_l: 0.0,
            n,
        };

        dendro.recompute_all();
        Ok(dendro)
    }

    /// Recompute all statistics from scratch (subtree sizes, edge counts, likelihood).
    #[allow(clippy::needless_range_loop)]
    fn recompute_all(&mut self) {
        let num_internal = self.n - 1;

        // Compute subtree sizes bottom-up
        for i in (0..num_internal).rev() {
            let nl = self.subtree_size(self.nodes[i].left);
            let nr = self.subtree_size(self.nodes[i].right);
            self.nodes[i].n = nl + nr;
        }

        // Compute labels (minimum leaf in subtree)
        for i in (0..num_internal).rev() {
            let ll = self.min_label(self.nodes[i].left);
            let rl = self.min_label(self.nodes[i].right);
            self.nodes[i].label = ll.min(rl);
        }

        // Compute edge counts using leaf enumeration + adjacency
        for i in 0..num_internal {
            let left_leaves = self.collect_leaves(self.nodes[i].left);
            let right_leaves = self.collect_leaves(self.nodes[i].right);
            let mut e = 0u32;
            for &lv in &left_leaves {
                for &rv in &right_leaves {
                    if self.adj[lv as usize].contains(&rv) {
                        e += 1;
                    }
                }
            }
            self.nodes[i].e = e;
        }

        // Compute probabilities and log-likelihood
        self.refresh_likelihood();
    }

    /// Refresh log-likelihood and probabilities from current e and n values.
    fn refresh_likelihood(&mut self) {
        self.total_log_l = 0.0;
        let num_internal = self.n - 1;
        for i in 0..num_internal {
            let nl = self.subtree_size(self.nodes[i].left);
            let nr = self.subtree_size(self.nodes[i].right);
            #[allow(clippy::cast_possible_truncation)]
            let nl_nr = (nl as u64) * (nr as u64);
            let ei = self.nodes[i].e as u64;

            if nl_nr == 0 {
                self.nodes[i].p = 0.0;
                self.nodes[i].log_l = 0.0;
            } else {
                #[allow(clippy::cast_precision_loss)]
                let p = ei as f64 / nl_nr as f64;
                self.nodes[i].p = p;

                if ei == 0 || ei == nl_nr {
                    self.nodes[i].log_l = 0.0;
                } else {
                    #[allow(clippy::cast_precision_loss)]
                    let dl = (ei as f64) * p.ln() + ((nl_nr - ei) as f64) * (1.0 - p).ln();
                    self.nodes[i].log_l = dl;
                }
            }
            self.total_log_l += self.nodes[i].log_l;
        }
    }

    fn subtree_size(&self, child: Child) -> u32 {
        match child {
            Child::Leaf(_) => 1,
            Child::Internal(idx) => self.nodes[idx].n,
        }
    }

    fn min_label(&self, child: Child) -> u32 {
        match child {
            Child::Leaf(v) => v,
            Child::Internal(idx) => self.nodes[idx].label,
        }
    }

    fn collect_leaves(&self, child: Child) -> Vec<u32> {
        let mut result = Vec::new();
        let mut stack = vec![child];
        while let Some(c) = stack.pop() {
            match c {
                Child::Leaf(v) => result.push(v),
                Child::Internal(idx) => {
                    stack.push(self.nodes[idx].left);
                    stack.push(self.nodes[idx].right);
                }
            }
        }
        result
    }

    /// Count edges between two subtrees.
    fn compute_edge_count(&self, a: Child, b: Child) -> u32 {
        let leaves_a = self.collect_leaves(a);
        let leaves_b = self.collect_leaves(b);
        let mut count = 0u32;
        // Use the smaller set for outer loop
        if leaves_a.len() <= leaves_b.len() {
            for &va in &leaves_a {
                for &vb in &leaves_b {
                    if self.adj[va as usize].contains(&vb) {
                        count += 1;
                    }
                }
            }
        } else {
            for &vb in &leaves_b {
                for &va in &leaves_a {
                    if self.adj[va as usize].contains(&vb) {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    /// Compute log-likelihood for given (e, n_L*n_R) pair.
    #[allow(clippy::cast_precision_loss)]
    fn node_log_likelihood(e: u32, nl_nr: u64) -> f64 {
        let ei = e as u64;
        if ei == 0 || ei == nl_nr || nl_nr == 0 {
            return 0.0;
        }
        let p = ei as f64 / nl_nr as f64;
        (ei as f64) * p.ln() + ((nl_nr - ei) as f64) * (1.0 - p).ln()
    }

    /// Perform a single MCMC move (tree rotation).
    fn mcmc_move(&mut self, rng: &mut SplitMix64) -> f64 {
        let num_internal = self.n - 1;
        if num_internal < 2 {
            return 0.0;
        }

        // Pick a random non-root internal node y, get its parent x
        let (x, y) = loop {
            let idx = rng.gen_index(num_internal);
            if let Some(p) = self.nodes[idx].parent {
                break (p, idx);
            }
        };

        // Determine which side y is on
        let side = match self.nodes[x].left {
            Child::Internal(c) if c == y => LEFT,
            _ => RIGHT,
        };

        if side == LEFT {
            if rng.gen_unit() < 0.5 {
                self.try_left_alpha(x, y, rng)
            } else {
                self.try_left_beta(x, y, rng)
            }
        } else if rng.gen_unit() < 0.5 {
            self.try_right_alpha(x, y, rng)
        } else {
            self.try_right_beta(x, y, rng)
        }
    }

    /// LEFT ALPHA: ((i,j),k) -> ((i,k),j)
    #[allow(clippy::many_single_char_names)]
    fn try_left_alpha(&mut self, x: usize, y: usize, rng: &mut SplitMix64) -> f64 {
        let i = self.nodes[y].left;
        let j = self.nodes[y].right;
        let k = self.nodes[x].right;

        let n_i = self.subtree_size(i);
        let n_j = self.subtree_size(j);
        let n_k = self.subtree_size(k);

        // New y: (i, k) -> e_ik, n_y = n_i * n_k
        let e_y = self.compute_edge_count(i, k);
        let nl_nr_y = (n_i as u64) * (n_k as u64);
        let log_l_y = Self::node_log_likelihood(e_y, nl_nr_y);

        // New x: (y_new, j) -> e_x = old_e_x + old_e_y - e_ik
        let e_x = self.nodes[x].e + self.nodes[y].e - e_y;
        let nl_nr_x = ((n_i + n_k) as u64) * (n_j as u64);
        let log_l_x = Self::node_log_likelihood(e_x, nl_nr_x);

        let d_log_l = (log_l_x - self.nodes[x].log_l) + (log_l_y - self.nodes[y].log_l);

        if d_log_l > 0.0 || rng.gen_unit() < d_log_l.exp() {
            // Accept: swap j and k
            self.nodes[y].right = k;
            self.nodes[x].right = j;
            self.update_child_parent(k, y, RIGHT);
            self.update_child_parent(j, x, RIGHT);
            self.nodes[y].n = n_i + n_k;
            self.nodes[y].e = e_y;
            #[allow(clippy::cast_precision_loss)]
            {
                self.nodes[y].p = e_y as f64 / nl_nr_y.max(1) as f64;
            }
            self.nodes[y].log_l = log_l_y;
            self.nodes[x].e = e_x;
            #[allow(clippy::cast_precision_loss)]
            {
                self.nodes[x].p = e_x as f64 / nl_nr_x.max(1) as f64;
            }
            self.nodes[x].log_l = log_l_x;
            self.nodes[x].n = n_i + n_k + n_j;
            self.total_log_l += d_log_l;
            d_log_l
        } else {
            0.0
        }
    }

    /// LEFT BETA: ((i,j),k) -> (i,(j,k))
    #[allow(clippy::many_single_char_names)]
    fn try_left_beta(&mut self, x: usize, y: usize, rng: &mut SplitMix64) -> f64 {
        let i = self.nodes[y].left;
        let j = self.nodes[y].right;
        let k = self.nodes[x].right;

        let n_i = self.subtree_size(i);
        let n_j = self.subtree_size(j);
        let n_k = self.subtree_size(k);

        // New y: (j, k) -> e_jk
        let e_y = self.compute_edge_count(j, k);
        let nl_nr_y = (n_j as u64) * (n_k as u64);
        let log_l_y = Self::node_log_likelihood(e_y, nl_nr_y);

        // New x: (i, y_new) -> e_x = old_e_x + old_e_y - e_jk
        let e_x = self.nodes[x].e + self.nodes[y].e - e_y;
        let nl_nr_x = (n_i as u64) * ((n_j + n_k) as u64);
        let log_l_x = Self::node_log_likelihood(e_x, nl_nr_x);

        let d_log_l = (log_l_x - self.nodes[x].log_l) + (log_l_y - self.nodes[y].log_l);

        if d_log_l > 0.0 || rng.gen_unit() < d_log_l.exp() {
            // Accept: restructure
            // y becomes right child of x; y holds (j, k); x holds (i, y)
            self.nodes[y].left = j;
            self.nodes[y].right = k;
            self.nodes[x].left = i;
            self.nodes[x].right = Child::Internal(y);
            self.update_child_parent(j, y, LEFT);
            self.update_child_parent(k, y, RIGHT);
            self.update_child_parent(i, x, LEFT);
            // y is already child of x, update edge list side

            self.nodes[y].n = n_j + n_k;
            self.nodes[y].e = e_y;
            #[allow(clippy::cast_precision_loss)]
            {
                self.nodes[y].p = e_y as f64 / nl_nr_y.max(1) as f64;
            }
            self.nodes[y].log_l = log_l_y;
            self.nodes[x].e = e_x;
            #[allow(clippy::cast_precision_loss)]
            {
                self.nodes[x].p = e_x as f64 / nl_nr_x.max(1) as f64;
            }
            self.nodes[x].log_l = log_l_x;
            self.nodes[x].n = n_i + n_j + n_k;
            self.total_log_l += d_log_l;
            d_log_l
        } else {
            0.0
        }
    }

    /// RIGHT ALPHA: (i,(j,k)) -> ((i,k),j)
    #[allow(clippy::many_single_char_names)]
    fn try_right_alpha(&mut self, x: usize, y: usize, rng: &mut SplitMix64) -> f64 {
        let i = self.nodes[x].left;
        let j = self.nodes[y].left;
        let k = self.nodes[y].right;

        let n_i = self.subtree_size(i);
        let n_j = self.subtree_size(j);
        let n_k = self.subtree_size(k);

        // New y: (i, k)
        let e_y = self.compute_edge_count(i, k);
        let nl_nr_y = (n_i as u64) * (n_k as u64);
        let log_l_y = Self::node_log_likelihood(e_y, nl_nr_y);

        // New x: (y_new, j)
        let e_x = self.nodes[x].e + self.nodes[y].e - e_y;
        let nl_nr_x = ((n_i + n_k) as u64) * (n_j as u64);
        let log_l_x = Self::node_log_likelihood(e_x, nl_nr_x);

        let d_log_l = (log_l_x - self.nodes[x].log_l) + (log_l_y - self.nodes[y].log_l);

        if d_log_l > 0.0 || rng.gen_unit() < d_log_l.exp() {
            // Accept: y becomes left child of x; y holds (i,k); x holds (y,j)
            self.nodes[y].left = i;
            self.nodes[y].right = k;
            self.nodes[x].left = Child::Internal(y);
            self.nodes[x].right = j;
            self.update_child_parent(i, y, LEFT);
            self.update_child_parent(k, y, RIGHT);
            self.update_child_parent(j, x, RIGHT);

            self.nodes[y].n = n_i + n_k;
            self.nodes[y].e = e_y;
            #[allow(clippy::cast_precision_loss)]
            {
                self.nodes[y].p = e_y as f64 / nl_nr_y.max(1) as f64;
            }
            self.nodes[y].log_l = log_l_y;
            self.nodes[x].e = e_x;
            #[allow(clippy::cast_precision_loss)]
            {
                self.nodes[x].p = e_x as f64 / nl_nr_x.max(1) as f64;
            }
            self.nodes[x].log_l = log_l_x;
            self.nodes[x].n = n_i + n_k + n_j;
            self.total_log_l += d_log_l;
            d_log_l
        } else {
            0.0
        }
    }

    /// RIGHT BETA: (i,(j,k)) -> ((i,j),k)
    #[allow(clippy::many_single_char_names)]
    fn try_right_beta(&mut self, x: usize, y: usize, rng: &mut SplitMix64) -> f64 {
        let i = self.nodes[x].left;
        let j = self.nodes[y].left;
        let k = self.nodes[y].right;

        let n_i = self.subtree_size(i);
        let n_j = self.subtree_size(j);
        let n_k = self.subtree_size(k);

        // New y: (i, j)
        let e_y = self.compute_edge_count(i, j);
        let nl_nr_y = (n_i as u64) * (n_j as u64);
        let log_l_y = Self::node_log_likelihood(e_y, nl_nr_y);

        // New x: (y_new, k)
        let e_x = self.nodes[x].e + self.nodes[y].e - e_y;
        let nl_nr_x = ((n_i + n_j) as u64) * (n_k as u64);
        let log_l_x = Self::node_log_likelihood(e_x, nl_nr_x);

        let d_log_l = (log_l_x - self.nodes[x].log_l) + (log_l_y - self.nodes[y].log_l);

        if d_log_l > 0.0 || rng.gen_unit() < d_log_l.exp() {
            // Accept: y becomes left child of x; y holds (i,j); x holds (y,k)
            self.nodes[y].left = i;
            self.nodes[y].right = j;
            self.nodes[x].left = Child::Internal(y);
            self.nodes[x].right = k;
            self.update_child_parent(i, y, LEFT);
            self.update_child_parent(j, y, RIGHT);
            self.update_child_parent(k, x, RIGHT);

            self.nodes[y].n = n_i + n_j;
            self.nodes[y].e = e_y;
            #[allow(clippy::cast_precision_loss)]
            {
                self.nodes[y].p = e_y as f64 / nl_nr_y.max(1) as f64;
            }
            self.nodes[y].log_l = log_l_y;
            self.nodes[x].e = e_x;
            #[allow(clippy::cast_precision_loss)]
            {
                self.nodes[x].p = e_x as f64 / nl_nr_x.max(1) as f64;
            }
            self.nodes[x].log_l = log_l_x;
            self.nodes[x].n = n_i + n_j + n_k;
            self.total_log_l += d_log_l;
            d_log_l
        } else {
            0.0
        }
    }

    /// Update the parent pointer of a child (leaf or internal).
    fn update_child_parent(&mut self, child: Child, new_parent: usize, side: u8) {
        match child {
            Child::Leaf(v) => {
                self.leaf_parent[v as usize] = new_parent;
                self.leaf_side[v as usize] = side;
            }
            Child::Internal(idx) => {
                self.nodes[idx].parent = Some(new_parent);
            }
        }
    }

    /// Export current dendrogram to an [`HrgTree`].
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    fn export_hrg(&self) -> HrgTree {
        let n = self.n as u32;
        let mut hrg = HrgTree::new(n);
        let num_internal = self.n - 1;

        for i in 0..num_internal {
            hrg.left[i] = match self.nodes[i].left {
                Child::Leaf(v) => v as i32,
                Child::Internal(idx) => -(idx as i32 + 1),
            };
            hrg.right[i] = match self.nodes[i].right {
                Child::Leaf(v) => v as i32,
                Child::Internal(idx) => -(idx as i32 + 1),
            };
            hrg.prob[i] = self.nodes[i].p;
            hrg.vertices[i] = self.nodes[i].n as i32;
            hrg.edges[i] = self.nodes[i].e as i32;
        }
        hrg
    }

    /// Get the accumulated split probability for each non-existing edge.
    #[allow(clippy::needless_range_loop, clippy::cast_possible_truncation)]
    fn predict_edges(&self, adj_counts: &[Vec<f64>], num_samples: f64) -> Vec<(u32, u32, f64)> {
        let n = self.n;
        let mut predictions = Vec::new();
        for i in 0..n {
            for j in (i + 1)..n {
                let iv = i as u32;
                let jv = j as u32;
                if !self.adj[i].contains(&jv) {
                    let prob = adj_counts[i][j] / num_samples;
                    predictions.push((iv, jv, prob));
                }
            }
        }
        predictions.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        predictions
    }

    /// Accumulate connection probabilities for current tree into counts matrix.
    #[allow(clippy::needless_range_loop, clippy::cast_possible_truncation)]
    fn accumulate_probabilities(&self, counts: &mut [Vec<f64>]) {
        for i in 0..self.n {
            for j in (i + 1)..self.n {
                let lca = self.find_lca(i as u32, j as u32);
                counts[i][j] += self.nodes[lca].p;
                counts[j][i] += self.nodes[lca].p;
            }
        }
    }

    /// Find LCA of two leaves.
    fn find_lca(&self, leaf_a: u32, leaf_b: u32) -> usize {
        let num_internal = self.n - 1;
        let mut visited = vec![false; num_internal];

        // Walk from leaf_a to root, marking
        let mut cur = self.leaf_parent[leaf_a as usize];
        loop {
            visited[cur] = true;
            match self.nodes[cur].parent {
                Some(p) => cur = p,
                None => break,
            }
        }

        // Walk from leaf_b to root, find first marked
        let mut cur = self.leaf_parent[leaf_b as usize];
        loop {
            if visited[cur] {
                return cur;
            }
            match self.nodes[cur].parent {
                Some(p) => cur = p,
                None => return cur, // root
            }
        }
    }
}

/// Build a symmetric adjacency list from a graph (ignoring direction and self-loops).
#[allow(clippy::cast_possible_truncation)]
fn build_adjacency(graph: &Graph) -> IgraphResult<Vec<Vec<u32>>> {
    let n = graph.vcount() as usize;
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];

    for eid in 0..graph.ecount() {
        let (from, to) = graph.edge(eid as u32)?;
        if from == to {
            continue;
        }
        if !adj[from as usize].contains(&to) {
            adj[from as usize].push(to);
        }
        if !adj[to as usize].contains(&from) {
            adj[to as usize].push(from);
        }
    }

    Ok(adj)
}

// ── Public API ──────────────────────────────────────────────────────────

/// Fit a hierarchical random graph model to a network using MCMC.
///
/// If `start_hrg` is provided, the MCMC begins from that dendrogram
/// structure; otherwise a random initial tree is used.
///
/// `steps` controls the MCMC budget:
/// - If `steps > 0`, exactly that many MCMC moves are performed.
/// - If `steps == 0`, the chain runs until convergence (mean
///   log-likelihood stabilizes over 65536-step windows).
///
/// # Errors
///
/// Returns an error if the graph has fewer than 3 vertices.
///
/// # Example
///
/// ```
/// use rust_igraph::{Graph, hrg_fit};
///
/// let g = Graph::from_edges(
///     &[(0,1),(1,2),(2,3),(3,4),(4,0),(0,2),(1,3)],
///     false, Some(5)
/// ).unwrap();
/// let hrg = hrg_fit(&g, None, 1000, 42).unwrap();
/// assert_eq!(hrg.size(), 5);
/// ```
pub fn hrg_fit(
    graph: &Graph,
    start_hrg: Option<&HrgTree>,
    steps: u64,
    seed: u64,
) -> IgraphResult<HrgTree> {
    let mut rng = SplitMix64::new(seed);

    let mut dendro = match start_hrg {
        Some(hrg) => Dendro::from_hrg(graph, hrg)?,
        None => Dendro::from_graph(graph, &mut rng)?,
    };

    if steps > 0 {
        // Fixed number of steps
        let mut best_l = dendro.total_log_l;
        let mut best_hrg = dendro.export_hrg();

        for _ in 0..steps {
            dendro.mcmc_move(&mut rng);
            if dendro.total_log_l > best_l {
                best_l = dendro.total_log_l;
                best_hrg = dendro.export_hrg();
            }
        }
        // Periodically refresh to correct FP drift
        dendro.refresh_likelihood();
        if dendro.total_log_l > best_l {
            best_hrg = dendro.export_hrg();
        }
        Ok(best_hrg)
    } else {
        // Run until convergence
        mcmc_equilibrium(&mut dendro, &mut rng);
        Ok(dendro.export_hrg())
    }
}

/// Run MCMC until convergence (mean log-likelihood stabilizes).
fn mcmc_equilibrium(dendro: &mut Dendro, rng: &mut SplitMix64) {
    let window = 65536u64;
    let mut old_mean = f64::NEG_INFINITY;

    loop {
        let mut sum = 0.0;
        for _ in 0..window {
            dendro.mcmc_move(rng);
            sum += dendro.total_log_l;
        }
        dendro.refresh_likelihood();

        #[allow(clippy::cast_precision_loss)]
        let new_mean = sum / window as f64;

        if (new_mean - old_mean).abs() < 1.0 {
            break;
        }
        old_mean = new_mean;
    }
}

/// Predict missing edges based on HRG ensemble sampling.
///
/// Returns a list of `(from, to, probability)` tuples sorted by
/// probability (highest first). Only non-existing edges are included.
///
/// The MCMC chain first runs a burn-in of `200*n` steps, then samples
/// with probability `1/(50*n)` per step until `num_samples` trees have
/// been collected.
///
/// # Errors
///
/// Returns an error if the graph has fewer than 3 vertices.
///
/// # Example
///
/// ```
/// use rust_igraph::{Graph, hrg_predict};
///
/// let g = Graph::from_edges(
///     &[(0,1),(1,2),(2,3),(3,4),(4,0)],
///     false, Some(5)
/// ).unwrap();
/// let predictions = hrg_predict(&g, None, 10, 42).unwrap();
/// assert!(!predictions.is_empty());
/// for &(from, to, prob) in &predictions {
///     assert!(prob >= 0.0 && prob <= 1.0);
///     assert!(from < to);
/// }
/// ```
#[allow(clippy::cast_precision_loss)]
pub fn hrg_predict(
    graph: &Graph,
    start_hrg: Option<&HrgTree>,
    num_samples: u64,
    seed: u64,
) -> IgraphResult<Vec<(u32, u32, f64)>> {
    let n = graph.vcount() as usize;
    let mut rng = SplitMix64::new(seed);

    let mut dendro = match start_hrg {
        Some(hrg) => Dendro::from_hrg(graph, hrg)?,
        None => Dendro::from_graph(graph, &mut rng)?,
    };

    // Burn-in phase
    let burn_in = 200 * n;
    for _ in 0..burn_in {
        dendro.mcmc_move(&mut rng);
    }
    dendro.refresh_likelihood();

    // Sample and accumulate probabilities
    let mut counts: Vec<Vec<f64>> = vec![vec![0.0; n]; n];
    let sample_interval = 50 * n;
    let mut samples_taken = 0u64;

    while samples_taken < num_samples {
        for _ in 0..sample_interval {
            dendro.mcmc_move(&mut rng);
        }
        dendro.accumulate_probabilities(&mut counts);
        samples_taken += 1;
    }

    let result = dendro.predict_edges(&counts, num_samples as f64);
    Ok(result)
}

/// Compute a consensus tree from MCMC samples of HRG models.
///
/// Returns `(parents, weights)`:
/// - `parents[i]` is the parent of vertex i in the consensus tree
///   (-1 for root). Vertex IDs 0..n are leaves, n..2n-1 are internal.
/// - `weights[i]` is the frequency (0..1) of each internal split.
///
/// # Errors
///
/// Returns an error if the graph has fewer than 3 vertices.
///
/// # Example
///
/// ```
/// use rust_igraph::{Graph, hrg_consensus};
///
/// let g = Graph::from_edges(
///     &[(0,1),(1,2),(2,3),(3,4),(4,0),(0,2)],
///     false, Some(5)
/// ).unwrap();
/// let (parents, weights) = hrg_consensus(&g, None, 10, 42).unwrap();
/// assert_eq!(parents.len(), 2 * 5 - 1);
/// assert_eq!(weights.len(), 4); // n-1 internal nodes
/// ```
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss
)]
pub fn hrg_consensus(
    graph: &Graph,
    start_hrg: Option<&HrgTree>,
    num_samples: u64,
    seed: u64,
) -> IgraphResult<(Vec<i32>, Vec<f64>)> {
    let n = graph.vcount() as usize;
    let mut rng = SplitMix64::new(seed);

    let mut dendro = match start_hrg {
        Some(hrg) => Dendro::from_hrg(graph, hrg)?,
        None => Dendro::from_graph(graph, &mut rng)?,
    };

    // Burn-in
    let burn_in = 200 * n;
    for _ in 0..burn_in {
        dendro.mcmc_move(&mut rng);
    }
    dendro.refresh_likelihood();

    // Sample split frequencies
    let mut split_counts: std::collections::HashMap<Vec<u32>, u64> =
        std::collections::HashMap::new();

    let sample_interval = 50 * n;
    let num_internal = n - 1;
    let mut samples_taken = 0u64;

    while samples_taken < num_samples {
        for _ in 0..sample_interval {
            dendro.mcmc_move(&mut rng);
        }

        // Record splits from current tree
        for i in 0..num_internal {
            let mut left_leaves = dendro.collect_leaves(dendro.nodes[i].left);
            left_leaves.sort_unstable();
            *split_counts.entry(left_leaves).or_insert(0) += 1;
        }
        samples_taken += 1;
    }

    // Build consensus tree from most frequent splits
    let mut splits: Vec<(Vec<u32>, u64)> = split_counts.into_iter().collect();
    splits.sort_by(|a, b| b.1.cmp(&a.1));

    // Build the consensus tree as parent array
    let total_nodes = 2 * n - 1;
    let mut parents = vec![-1i32; total_nodes];
    let mut weights = vec![0.0f64; n - 1];

    // Start with all leaves under root (internal node index 0 = position n)
    let root_pos = n;
    for i in 0..n {
        parents[i] = root_pos as i32;
    }

    // Greedily assign the top compatible splits
    let mut used_internal = 0usize;

    for (split, count) in splits.iter().take(n - 1) {
        if split.len() < 2 || split.len() >= n {
            continue;
        }
        if used_internal >= n - 1 {
            break;
        }

        let internal_idx = n + used_internal;
        weights[used_internal] = *count as f64 / (num_samples * num_internal as u64) as f64;

        // Assign leaves in split to this internal node
        for &leaf in split {
            parents[leaf as usize] = internal_idx as i32;
        }

        // This internal node is child of root
        if used_internal > 0 {
            parents[internal_idx] = root_pos as i32;
        }

        used_internal += 1;
    }

    Ok((parents, weights))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_graph() -> Graph {
        // Pentagon with one diagonal
        Graph::from_edges(
            &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 0), (0, 2)],
            false,
            Some(5),
        )
        .expect("graph creation")
    }

    #[test]
    fn hrg_fit_returns_correct_size() {
        let g = make_test_graph();
        let hrg = hrg_fit(&g, None, 500, 42).expect("hrg_fit");
        assert_eq!(hrg.size(), 5);
        assert_eq!(hrg.num_internal(), 4);
    }

    #[test]
    fn hrg_fit_deterministic() {
        let g = make_test_graph();
        let h1 = hrg_fit(&g, None, 200, 99).expect("hrg_fit");
        let h2 = hrg_fit(&g, None, 200, 99).expect("hrg_fit");
        for i in 0..h1.num_internal() {
            assert_eq!(h1.left[i], h2.left[i]);
            assert_eq!(h1.right[i], h2.right[i]);
            assert!((h1.prob[i] - h2.prob[i]).abs() < 1e-10);
        }
    }

    #[test]
    fn hrg_fit_rejects_small_graph() {
        let g = Graph::from_edges(&[(0, 1)], false, Some(2)).expect("graph");
        assert!(hrg_fit(&g, None, 100, 0).is_err());
    }

    #[test]
    fn hrg_fit_from_start_hrg() {
        let g = make_test_graph();
        let h1 = hrg_fit(&g, None, 200, 42).expect("hrg_fit");
        let h2 = hrg_fit(&g, Some(&h1), 200, 77).expect("hrg_fit from start");
        assert_eq!(h2.size(), 5);
    }

    #[test]
    fn hrg_fit_probs_valid() {
        let g = make_test_graph();
        let hrg = hrg_fit(&g, None, 500, 42).expect("hrg_fit");
        for i in 0..hrg.num_internal() {
            assert!(hrg.prob[i] >= 0.0 && hrg.prob[i] <= 1.0);
        }
    }

    #[test]
    fn hrg_predict_returns_results() {
        let g = make_test_graph();
        let preds = hrg_predict(&g, None, 20, 42).expect("hrg_predict");
        // Pentagon+diagonal has 6 edges out of 10 possible, so 4 missing
        assert_eq!(preds.len(), 4);
        for &(from, to, prob) in &preds {
            assert!(from < to);
            assert!((0.0..=1.0).contains(&prob));
        }
    }

    #[test]
    fn hrg_predict_sorted_by_prob() {
        let g = make_test_graph();
        let preds = hrg_predict(&g, None, 20, 42).expect("hrg_predict");
        for w in preds.windows(2) {
            assert!(w[0].2 >= w[1].2);
        }
    }

    #[test]
    fn hrg_consensus_returns_valid_tree() {
        let g = make_test_graph();
        let (parents, weights) = hrg_consensus(&g, None, 20, 42).expect("hrg_consensus");
        assert_eq!(parents.len(), 9); // 2*5-1
        assert_eq!(weights.len(), 4); // n-1
        // Root should have parent -1
        assert!(parents.contains(&-1));
        for &w in &weights {
            assert!((0.0..=1.0).contains(&w));
        }
    }

    #[test]
    fn dendro_from_graph_correct_structure() {
        let g = make_test_graph();
        let mut rng = SplitMix64::new(42);
        let d = Dendro::from_graph(&g, &mut rng).expect("dendro");
        assert_eq!(d.n, 5);
        assert_eq!(d.nodes.len(), 4); // n-1 internal nodes
        // Root's subtree should have all n vertices
        assert_eq!(d.nodes[0].n, 5);
    }

    #[test]
    fn dendro_likelihood_finite() {
        let g = make_test_graph();
        let mut rng = SplitMix64::new(42);
        let d = Dendro::from_graph(&g, &mut rng).expect("dendro");
        assert!(d.total_log_l.is_finite());
        assert!(d.total_log_l <= 0.0);
    }

    #[test]
    fn dendro_mcmc_move_changes_likelihood() {
        let g = make_test_graph();
        let mut rng = SplitMix64::new(42);
        let mut d = Dendro::from_graph(&g, &mut rng).expect("dendro");
        let initial_l = d.total_log_l;
        let mut changed = false;
        for _ in 0..100 {
            d.mcmc_move(&mut rng);
            if (d.total_log_l - initial_l).abs() > 1e-15 {
                changed = true;
                break;
            }
        }
        assert!(changed, "MCMC should change likelihood within 100 moves");
    }

    #[test]
    fn dendro_export_roundtrip() {
        let g = make_test_graph();
        let mut rng = SplitMix64::new(42);
        let d = Dendro::from_graph(&g, &mut rng).expect("dendro");
        let hrg = d.export_hrg();
        assert_eq!(hrg.size(), 5);
        assert_eq!(hrg.num_internal(), 4);
    }
}
