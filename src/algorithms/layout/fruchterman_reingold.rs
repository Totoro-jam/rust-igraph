//! Fruchterman-Reingold force-directed layout (ALGO-LO-002).

use crate::algorithms::connectivity::connected_components;
use crate::core::{Graph, IgraphError, IgraphResult, rng::SplitMix64};

/// Grid acceleration mode for Fruchterman-Reingold layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrGrid {
    /// Use grid-based acceleration (faster but less accurate for large graphs).
    Grid,
    /// No grid — compute all-pairs repulsion (O(V²) per iteration).
    NoGrid,
    /// Automatically choose: grid for V > 1000, otherwise no grid.
    Auto,
}

/// Optional per-vertex bounding box constraints.
#[derive(Debug, Clone, Default)]
pub struct FrBounds {
    /// Minimum x coordinate per vertex (`None` = unbounded).
    pub minx: Option<Vec<f64>>,
    /// Maximum x coordinate per vertex (`None` = unbounded).
    pub maxx: Option<Vec<f64>>,
    /// Minimum y coordinate per vertex (`None` = unbounded).
    pub miny: Option<Vec<f64>>,
    /// Maximum y coordinate per vertex (`None` = unbounded).
    pub maxy: Option<Vec<f64>>,
}

/// Parameters for the 2D Fruchterman-Reingold layout.
#[derive(Debug, Clone)]
pub struct FrParams {
    /// Number of iterations (default: 500).
    pub niter: u32,
    /// Starting temperature — maximum displacement per step (default: sqrt(V)).
    pub start_temp: Option<f64>,
    /// Grid acceleration mode (default: Auto).
    pub grid: FrGrid,
    /// Edge weights (must be positive). `None` means all weights = 1.
    pub weights: Option<Vec<f64>>,
    /// Per-vertex coordinate bounds.
    pub bounds: FrBounds,
    /// If `Some(coords)`, use as initial layout seed. Otherwise random init.
    pub seed_coords: Option<Vec<(f64, f64)>>,
    /// RNG seed for random initialization (default: 42).
    pub rng_seed: u64,
}

impl Default for FrParams {
    fn default() -> Self {
        Self {
            niter: 500,
            start_temp: None,
            grid: FrGrid::Auto,
            weights: None,
            bounds: FrBounds::default(),
            seed_coords: None,
            rng_seed: 42,
        }
    }
}

/// 3D bounding box constraints.
#[derive(Debug, Clone, Default)]
pub struct FrBounds3d {
    /// Minimum x coordinate per vertex.
    pub minx: Option<Vec<f64>>,
    /// Maximum x coordinate per vertex.
    pub maxx: Option<Vec<f64>>,
    /// Minimum y coordinate per vertex.
    pub miny: Option<Vec<f64>>,
    /// Maximum y coordinate per vertex.
    pub maxy: Option<Vec<f64>>,
    /// Minimum z coordinate per vertex.
    pub minz: Option<Vec<f64>>,
    /// Maximum z coordinate per vertex.
    pub maxz: Option<Vec<f64>>,
}

/// Parameters for the 3D Fruchterman-Reingold layout.
#[derive(Debug, Clone)]
pub struct FrParams3d {
    /// Number of iterations (default: 500).
    pub niter: u32,
    /// Starting temperature (default: sqrt(V)).
    pub start_temp: Option<f64>,
    /// Edge weights (must be positive). `None` means unit weights.
    pub weights: Option<Vec<f64>>,
    /// Per-vertex 3D coordinate bounds.
    pub bounds: FrBounds3d,
    /// Initial layout seed. `None` means random init.
    pub seed_coords: Option<Vec<(f64, f64, f64)>>,
    /// RNG seed for random initialization (default: 42).
    pub rng_seed: u64,
}

impl Default for FrParams3d {
    fn default() -> Self {
        Self {
            niter: 500,
            start_temp: None,
            weights: None,
            bounds: FrBounds3d::default(),
            seed_coords: None,
            rng_seed: 42,
        }
    }
}

/// 2D Fruchterman-Reingold force-directed layout.
///
/// Simulates attractive force `f_a(d) = -w * d²` between connected vertices
/// and repulsive force `f_r(d) = 1/d` between all vertex pairs. Temperature
/// cools linearly from `start_temp` to 0 over `niter` iterations.
///
/// For disconnected graphs, a weak global attraction `n^(-3/2)` keeps
/// components from drifting apart.
///
/// Reference: Fruchterman & Reingold, "Graph Drawing by Force-directed
/// Placement", Software—Practice and Experience 21(11), 1991.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, layout_fruchterman_reingold, FrParams};
///
/// let mut g = Graph::with_vertices(5);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// g.add_edge(4, 0).unwrap();
///
/// let coords = layout_fruchterman_reingold(&g, &FrParams::default()).unwrap();
/// assert_eq!(coords.len(), 5);
/// ```
pub fn layout_fruchterman_reingold(
    graph: &Graph,
    params: &FrParams,
) -> IgraphResult<Vec<(f64, f64)>> {
    let vcount = graph.vcount() as usize;
    let ecount = graph.ecount() as usize;

    if let Some(ref w) = params.weights {
        if w.len() != ecount {
            return Err(IgraphError::InvalidArgument(
                "weight vector length must equal edge count".into(),
            ));
        }
        if ecount > 0 && w.iter().any(|&x| x <= 0.0) {
            return Err(IgraphError::InvalidArgument(
                "weights must be positive for Fruchterman-Reingold layout".into(),
            ));
        }
    }

    validate_bounds_2d(vcount, &params.bounds)?;

    if let Some(ref sc) = params.seed_coords {
        if sc.len() != vcount {
            return Err(IgraphError::InvalidArgument(
                "seed_coords length must equal vertex count".into(),
            ));
        }
    }

    if vcount == 0 {
        return Ok(Vec::new());
    }

    let start_temp = params.start_temp.unwrap_or_else(|| (vcount as f64).sqrt());

    let use_grid = match params.grid {
        FrGrid::Grid => true,
        FrGrid::NoGrid => false,
        FrGrid::Auto => vcount > 1000,
    };

    if use_grid {
        fr_grid_2d(graph, params, start_temp)
    } else {
        fr_naive_2d(graph, params, start_temp)
    }
}

/// 3D Fruchterman-Reingold force-directed layout.
///
/// Same algorithm as the 2D version but in three dimensions.
/// No grid acceleration is available for 3D.
///
/// # Examples
///
/// ```
/// use rust_igraph::{Graph, layout_fruchterman_reingold_3d, FrParams3d};
///
/// let mut g = Graph::with_vertices(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
///
/// let coords = layout_fruchterman_reingold_3d(&g, &FrParams3d::default()).unwrap();
/// assert_eq!(coords.len(), 4);
/// ```
pub fn layout_fruchterman_reingold_3d(
    graph: &Graph,
    params: &FrParams3d,
) -> IgraphResult<Vec<(f64, f64, f64)>> {
    let vcount = graph.vcount() as usize;
    let ecount = graph.ecount() as usize;

    if let Some(ref w) = params.weights {
        if w.len() != ecount {
            return Err(IgraphError::InvalidArgument(
                "weight vector length must equal edge count".into(),
            ));
        }
        if ecount > 0 && w.iter().any(|&x| x <= 0.0) {
            return Err(IgraphError::InvalidArgument(
                "weights must be positive for Fruchterman-Reingold layout".into(),
            ));
        }
    }

    validate_bounds_3d(vcount, &params.bounds)?;

    if let Some(ref sc) = params.seed_coords {
        if sc.len() != vcount {
            return Err(IgraphError::InvalidArgument(
                "seed_coords length must equal vertex count".into(),
            ));
        }
    }

    if vcount == 0 {
        return Ok(Vec::new());
    }

    let start_temp = params.start_temp.unwrap_or_else(|| (vcount as f64).sqrt());
    fr_naive_3d(graph, params, start_temp)
}

// -- Internal implementations --

fn fr_naive_2d(graph: &Graph, params: &FrParams, start_temp: f64) -> IgraphResult<Vec<(f64, f64)>> {
    let vcount = graph.vcount() as usize;
    let ecount = graph.ecount() as usize;

    let cc = connected_components(graph)?;
    let connected = cc.count == 1;
    let big_c = if connected {
        0.0
    } else {
        vcount as f64 * (vcount as f64).sqrt()
    };

    let mut pos: Vec<(f64, f64)> = if let Some(ref sc) = params.seed_coords {
        sc.clone()
    } else {
        init_random_bounded_2d(vcount, &params.bounds, params.rng_seed)
    };

    let mut rng = SplitMix64::new(params.rng_seed.wrapping_add(0xDEAD_BEEF));
    let mut dispx = vec![0.0f64; vcount];
    let mut dispy = vec![0.0f64; vcount];
    let niter = params.niter;
    let diff_temp = if niter > 0 {
        start_temp / niter as f64
    } else {
        0.0
    };
    let mut temp = start_temp;

    for _ in 0..niter {
        dispx.iter_mut().for_each(|d| *d = 0.0);
        dispy.iter_mut().for_each(|d| *d = 0.0);

        // Repulsive forces
        if connected {
            for v in 0..vcount {
                for u in (v + 1)..vcount {
                    let mut dx = pos[v].0 - pos[u].0;
                    let mut dy = pos[v].1 - pos[u].1;
                    let mut dlen = dx * dx + dy * dy;
                    while dlen == 0.0 {
                        dx = rng.gen_unit() * 2e-9 - 1e-9;
                        dy = rng.gen_unit() * 2e-9 - 1e-9;
                        dlen = dx * dx + dy * dy;
                    }
                    dispx[v] += dx / dlen;
                    dispy[v] += dy / dlen;
                    dispx[u] -= dx / dlen;
                    dispy[u] -= dy / dlen;
                }
            }
        } else {
            for v in 0..vcount {
                for u in (v + 1)..vcount {
                    let mut dx = pos[v].0 - pos[u].0;
                    let mut dy = pos[v].1 - pos[u].1;
                    let mut dlen = dx * dx + dy * dy;
                    while dlen == 0.0 {
                        dx = rng.gen_unit() * 2e-9 - 1e-9;
                        dy = rng.gen_unit() * 2e-9 - 1e-9;
                        dlen = dx * dx + dy * dy;
                    }
                    let rdlen = dlen.sqrt();
                    let factor = (big_c - dlen * rdlen) / (dlen * big_c);
                    dispx[v] += dx * factor;
                    dispy[v] += dy * factor;
                    dispx[u] -= dx * factor;
                    dispy[u] -= dy * factor;
                }
            }
        }

        // Attractive forces
        for e in 0..ecount {
            let (from, to) = graph.edge(e as u32)?;
            let v = from as usize;
            let u = to as usize;
            let dx = pos[v].0 - pos[u].0;
            let dy = pos[v].1 - pos[u].1;
            let w = params.weights.as_ref().map_or(1.0, |ws| ws[e]);
            let dlen = (dx * dx + dy * dy).sqrt() * w;
            dispx[v] -= dx * dlen;
            dispy[v] -= dy * dlen;
            dispx[u] += dx * dlen;
            dispy[u] += dy * dlen;
        }

        // Apply displacement capped by temperature
        for v in 0..vcount {
            let dx = dispx[v] + rng.gen_unit() * 2e-9 - 1e-9;
            let dy = dispy[v] + rng.gen_unit() * 2e-9 - 1e-9;
            let displen = (dx * dx + dy * dy).sqrt();
            let (cdx, cdy) = if displen > temp {
                (dx * temp / displen, dy * temp / displen)
            } else {
                (dx, dy)
            };
            if displen > 0.0 {
                pos[v].0 += cdx;
                pos[v].1 += cdy;
            }
            apply_bounds_2d(&mut pos[v], v, &params.bounds);
        }

        temp -= diff_temp;
    }

    Ok(pos)
}

fn fr_grid_2d(graph: &Graph, params: &FrParams, start_temp: f64) -> IgraphResult<Vec<(f64, f64)>> {
    let vcount = graph.vcount() as usize;
    let ecount = graph.ecount() as usize;
    let width = (vcount as f64).sqrt();
    let cellsize = 2.0f64;

    let mut pos: Vec<(f64, f64)> = if let Some(ref sc) = params.seed_coords {
        sc.clone()
    } else {
        init_random_bounded_2d(vcount, &params.bounds, params.rng_seed)
    };

    let mut rng = SplitMix64::new(params.rng_seed.wrapping_add(0xDEAD_BEEF));
    let mut dispx = vec![0.0f64; vcount];
    let mut dispy = vec![0.0f64; vcount];
    let niter = params.niter;
    let diff_temp = if niter > 0 {
        start_temp / niter as f64
    } else {
        0.0
    };
    let mut temp = start_temp;

    let half_w = width / 2.0;
    let cell_threshold_sq = cellsize * cellsize;

    for _ in 0..niter {
        dispx.iter_mut().for_each(|d| *d = 0.0);
        dispy.iter_mut().for_each(|d| *d = 0.0);

        // Grid-based repulsion: only consider pairs within cell distance
        let grid = Grid2d::build(&pos, -half_w, half_w, -half_w, half_w, cellsize);
        for v in 0..vcount {
            let neighbors = grid.nearby_vertices(v, &pos);
            for &u in &neighbors {
                if u <= v {
                    continue;
                }
                let mut dx = pos[v].0 - pos[u].0;
                let mut dy = pos[v].1 - pos[u].1;
                let mut dlen = dx * dx + dy * dy;
                while dlen == 0.0 {
                    dx = rng.gen_unit() * 2e-9 - 1e-9;
                    dy = rng.gen_unit() * 2e-9 - 1e-9;
                    dlen = dx * dx + dy * dy;
                }
                if dlen < cell_threshold_sq {
                    dispx[v] += dx / dlen;
                    dispy[v] += dy / dlen;
                    dispx[u] -= dx / dlen;
                    dispy[u] -= dy / dlen;
                }
            }
        }

        // Attractive forces
        for e in 0..ecount {
            let (from, to) = graph.edge(e as u32)?;
            let v = from as usize;
            let u = to as usize;
            let dx = pos[v].0 - pos[u].0;
            let dy = pos[v].1 - pos[u].1;
            let w = params.weights.as_ref().map_or(1.0, |ws| ws[e]);
            let dlen = (dx * dx + dy * dy).sqrt() * w;
            dispx[v] -= dx * dlen;
            dispy[v] -= dy * dlen;
            dispx[u] += dx * dlen;
            dispy[u] += dy * dlen;
        }

        // Apply displacement
        for v in 0..vcount {
            let dx = dispx[v] + rng.gen_unit() * 2e-9 - 1e-9;
            let dy = dispy[v] + rng.gen_unit() * 2e-9 - 1e-9;
            let displen = (dx * dx + dy * dy).sqrt();
            let (cdx, cdy) = if displen > temp {
                (dx * temp / displen, dy * temp / displen)
            } else {
                (dx, dy)
            };
            if displen > 0.0 {
                pos[v].0 += cdx;
                pos[v].1 += cdy;
            }
            apply_bounds_2d(&mut pos[v], v, &params.bounds);
        }

        temp -= diff_temp;
    }

    Ok(pos)
}

fn fr_naive_3d(
    graph: &Graph,
    params: &FrParams3d,
    start_temp: f64,
) -> IgraphResult<Vec<(f64, f64, f64)>> {
    let vcount = graph.vcount() as usize;
    let ecount = graph.ecount() as usize;

    let cc = connected_components(graph)?;
    let connected = cc.count == 1;
    let big_c = if connected {
        0.0
    } else {
        vcount as f64 * (vcount as f64).sqrt()
    };

    let mut pos: Vec<(f64, f64, f64)> = if let Some(ref sc) = params.seed_coords {
        sc.clone()
    } else {
        init_random_bounded_3d(vcount, &params.bounds, params.rng_seed)
    };

    let mut rng = SplitMix64::new(params.rng_seed.wrapping_add(0xDEAD_BEEF));
    let mut dispx = vec![0.0f64; vcount];
    let mut dispy = vec![0.0f64; vcount];
    let mut dispz = vec![0.0f64; vcount];
    let niter = params.niter;
    let diff_temp = if niter > 0 {
        start_temp / niter as f64
    } else {
        0.0
    };
    let mut temp = start_temp;

    for _ in 0..niter {
        dispx.iter_mut().for_each(|d| *d = 0.0);
        dispy.iter_mut().for_each(|d| *d = 0.0);
        dispz.iter_mut().for_each(|d| *d = 0.0);

        // Repulsive forces
        if connected {
            for v in 0..vcount {
                for u in (v + 1)..vcount {
                    let mut dx = pos[v].0 - pos[u].0;
                    let mut dy = pos[v].1 - pos[u].1;
                    let mut dz = pos[v].2 - pos[u].2;
                    let mut dlen = dx * dx + dy * dy + dz * dz;
                    while dlen == 0.0 {
                        dx = rng.gen_unit() * 2e-9 - 1e-9;
                        dy = rng.gen_unit() * 2e-9 - 1e-9;
                        dz = rng.gen_unit() * 2e-9 - 1e-9;
                        dlen = dx * dx + dy * dy + dz * dz;
                    }
                    dispx[v] += dx / dlen;
                    dispy[v] += dy / dlen;
                    dispz[v] += dz / dlen;
                    dispx[u] -= dx / dlen;
                    dispy[u] -= dy / dlen;
                    dispz[u] -= dz / dlen;
                }
            }
        } else {
            for v in 0..vcount {
                for u in (v + 1)..vcount {
                    let mut dx = pos[v].0 - pos[u].0;
                    let mut dy = pos[v].1 - pos[u].1;
                    let mut dz = pos[v].2 - pos[u].2;
                    let mut dlen = dx * dx + dy * dy + dz * dz;
                    while dlen == 0.0 {
                        dx = rng.gen_unit() * 2e-9 - 1e-9;
                        dy = rng.gen_unit() * 2e-9 - 1e-9;
                        dz = rng.gen_unit() * 2e-9 - 1e-9;
                        dlen = dx * dx + dy * dy + dz * dz;
                    }
                    let rdlen = dlen.sqrt();
                    let factor = (big_c - dlen * rdlen) / (dlen * big_c);
                    dispx[v] += dx * factor;
                    dispy[v] += dy * factor;
                    dispz[v] += dz * factor;
                    dispx[u] -= dx * factor;
                    dispy[u] -= dy * factor;
                    dispz[u] -= dz * factor;
                }
            }
        }

        // Attractive forces
        for e in 0..ecount {
            let (from, to) = graph.edge(e as u32)?;
            let v = from as usize;
            let u = to as usize;
            let dx = pos[v].0 - pos[u].0;
            let dy = pos[v].1 - pos[u].1;
            let dz = pos[v].2 - pos[u].2;
            let w = params.weights.as_ref().map_or(1.0, |ws| ws[e]);
            let dlen = (dx * dx + dy * dy + dz * dz).sqrt() * w;
            dispx[v] -= dx * dlen;
            dispy[v] -= dy * dlen;
            dispz[v] -= dz * dlen;
            dispx[u] += dx * dlen;
            dispy[u] += dy * dlen;
            dispz[u] += dz * dlen;
        }

        // Apply displacement
        for v in 0..vcount {
            let dx = dispx[v] + rng.gen_unit() * 2e-9 - 1e-9;
            let dy = dispy[v] + rng.gen_unit() * 2e-9 - 1e-9;
            let dz = dispz[v] + rng.gen_unit() * 2e-9 - 1e-9;
            let displen = (dx * dx + dy * dy + dz * dz).sqrt();
            let (cdx, cdy, cdz) = if displen > temp {
                let scale = temp / displen;
                (dx * scale, dy * scale, dz * scale)
            } else {
                (dx, dy, dz)
            };
            if displen > 0.0 {
                pos[v].0 += cdx;
                pos[v].1 += cdy;
                pos[v].2 += cdz;
            }
            apply_bounds_3d(&mut pos[v], v, &params.bounds);
        }

        temp -= diff_temp;
    }

    Ok(pos)
}

// -- Helpers --

fn validate_bounds_2d(vcount: usize, bounds: &FrBounds) -> IgraphResult<()> {
    if let Some(ref v) = bounds.minx {
        if v.len() != vcount {
            return Err(IgraphError::InvalidArgument("minx length mismatch".into()));
        }
    }
    if let Some(ref v) = bounds.maxx {
        if v.len() != vcount {
            return Err(IgraphError::InvalidArgument("maxx length mismatch".into()));
        }
    }
    if let Some(ref v) = bounds.miny {
        if v.len() != vcount {
            return Err(IgraphError::InvalidArgument("miny length mismatch".into()));
        }
    }
    if let Some(ref v) = bounds.maxy {
        if v.len() != vcount {
            return Err(IgraphError::InvalidArgument("maxy length mismatch".into()));
        }
    }
    if let (Some(lo), Some(hi)) = (&bounds.minx, &bounds.maxx) {
        if lo.iter().zip(hi.iter()).any(|(a, b)| a > b) {
            return Err(IgraphError::InvalidArgument(
                "minx must not exceed maxx".into(),
            ));
        }
    }
    if let (Some(lo), Some(hi)) = (&bounds.miny, &bounds.maxy) {
        if lo.iter().zip(hi.iter()).any(|(a, b)| a > b) {
            return Err(IgraphError::InvalidArgument(
                "miny must not exceed maxy".into(),
            ));
        }
    }
    Ok(())
}

fn validate_bounds_3d(vcount: usize, bounds: &FrBounds3d) -> IgraphResult<()> {
    for (name, vec) in [
        ("minx", &bounds.minx),
        ("maxx", &bounds.maxx),
        ("miny", &bounds.miny),
        ("maxy", &bounds.maxy),
        ("minz", &bounds.minz),
        ("maxz", &bounds.maxz),
    ] {
        if let Some(v) = vec {
            if v.len() != vcount {
                let msg = match name {
                    "minx" => "minx length mismatch",
                    "maxx" => "maxx length mismatch",
                    "miny" => "miny length mismatch",
                    "maxy" => "maxy length mismatch",
                    "minz" => "minz length mismatch",
                    _ => "maxz length mismatch",
                };
                return Err(IgraphError::InvalidArgument(msg.into()));
            }
        }
    }
    if let (Some(lo), Some(hi)) = (&bounds.minx, &bounds.maxx) {
        if lo.iter().zip(hi.iter()).any(|(a, b)| a > b) {
            return Err(IgraphError::InvalidArgument(
                "minx must not exceed maxx".into(),
            ));
        }
    }
    if let (Some(lo), Some(hi)) = (&bounds.miny, &bounds.maxy) {
        if lo.iter().zip(hi.iter()).any(|(a, b)| a > b) {
            return Err(IgraphError::InvalidArgument(
                "miny must not exceed maxy".into(),
            ));
        }
    }
    if let (Some(lo), Some(hi)) = (&bounds.minz, &bounds.maxz) {
        if lo.iter().zip(hi.iter()).any(|(a, b)| a > b) {
            return Err(IgraphError::InvalidArgument(
                "minz must not exceed maxz".into(),
            ));
        }
    }
    Ok(())
}

fn init_random_bounded_2d(vcount: usize, bounds: &FrBounds, seed: u64) -> Vec<(f64, f64)> {
    let mut rng = SplitMix64::new(seed);
    (0..vcount)
        .map(|i| {
            let lo_x = bounds.minx.as_ref().map_or(-1.0, |v| v[i]);
            let hi_x = bounds.maxx.as_ref().map_or(1.0, |v| v[i]);
            let lo_y = bounds.miny.as_ref().map_or(-1.0, |v| v[i]);
            let hi_y = bounds.maxy.as_ref().map_or(1.0, |v| v[i]);
            let x = lo_x + rng.gen_unit() * (hi_x - lo_x);
            let y = lo_y + rng.gen_unit() * (hi_y - lo_y);
            (x, y)
        })
        .collect()
}

fn init_random_bounded_3d(vcount: usize, bounds: &FrBounds3d, seed: u64) -> Vec<(f64, f64, f64)> {
    let mut rng = SplitMix64::new(seed);
    (0..vcount)
        .map(|i| {
            let lo_x = bounds.minx.as_ref().map_or(-1.0, |v| v[i]);
            let hi_x = bounds.maxx.as_ref().map_or(1.0, |v| v[i]);
            let lo_y = bounds.miny.as_ref().map_or(-1.0, |v| v[i]);
            let hi_y = bounds.maxy.as_ref().map_or(1.0, |v| v[i]);
            let lo_z = bounds.minz.as_ref().map_or(-1.0, |v| v[i]);
            let hi_z = bounds.maxz.as_ref().map_or(1.0, |v| v[i]);
            let x = lo_x + rng.gen_unit() * (hi_x - lo_x);
            let y = lo_y + rng.gen_unit() * (hi_y - lo_y);
            let z = lo_z + rng.gen_unit() * (hi_z - lo_z);
            (x, y, z)
        })
        .collect()
}

fn apply_bounds_2d(pos: &mut (f64, f64), i: usize, bounds: &FrBounds) {
    if let Some(ref v) = bounds.minx {
        if pos.0 < v[i] {
            pos.0 = v[i];
        }
    }
    if let Some(ref v) = bounds.maxx {
        if pos.0 > v[i] {
            pos.0 = v[i];
        }
    }
    if let Some(ref v) = bounds.miny {
        if pos.1 < v[i] {
            pos.1 = v[i];
        }
    }
    if let Some(ref v) = bounds.maxy {
        if pos.1 > v[i] {
            pos.1 = v[i];
        }
    }
}

fn apply_bounds_3d(pos: &mut (f64, f64, f64), i: usize, bounds: &FrBounds3d) {
    if let Some(ref v) = bounds.minx {
        if pos.0 < v[i] {
            pos.0 = v[i];
        }
    }
    if let Some(ref v) = bounds.maxx {
        if pos.0 > v[i] {
            pos.0 = v[i];
        }
    }
    if let Some(ref v) = bounds.miny {
        if pos.1 < v[i] {
            pos.1 = v[i];
        }
    }
    if let Some(ref v) = bounds.maxy {
        if pos.1 > v[i] {
            pos.1 = v[i];
        }
    }
    if let Some(ref v) = bounds.minz {
        if pos.2 < v[i] {
            pos.2 = v[i];
        }
    }
    if let Some(ref v) = bounds.maxz {
        if pos.2 > v[i] {
            pos.2 = v[i];
        }
    }
}

// Simple 2D spatial grid for acceleration
struct Grid2d {
    cells: Vec<Vec<usize>>,
    nx: usize,
    ny: usize,
    x_min: f64,
    y_min: f64,
    cellsize: f64,
}

impl Grid2d {
    fn build(
        pos: &[(f64, f64)],
        x_min: f64,
        x_max: f64,
        y_min: f64,
        y_max: f64,
        cellsize: f64,
    ) -> Self {
        let nx = ((x_max - x_min) / cellsize).ceil().max(1.0) as usize;
        let ny = ((y_max - y_min) / cellsize).ceil().max(1.0) as usize;
        let mut cells = vec![Vec::new(); nx * ny];
        for (i, &(x, y)) in pos.iter().enumerate() {
            let cx = ((x - x_min) / cellsize).floor() as isize;
            let cy = ((y - y_min) / cellsize).floor() as isize;
            let cx = cx.clamp(0, nx as isize - 1) as usize;
            let cy = cy.clamp(0, ny as isize - 1) as usize;
            cells[cy * nx + cx].push(i);
        }
        Grid2d {
            cells,
            nx,
            ny,
            x_min,
            y_min,
            cellsize,
        }
    }

    fn nearby_vertices(&self, v: usize, pos: &[(f64, f64)]) -> Vec<usize> {
        let (x, y) = pos[v];
        let cx = ((x - self.x_min) / self.cellsize).floor() as isize;
        let cy = ((y - self.y_min) / self.cellsize).floor() as isize;
        let mut result = Vec::new();
        for dy in -1..=1i64 {
            for dx in -1..=1i64 {
                let nx = cx + dx as isize;
                let ny = cy + dy as isize;
                if nx >= 0 && nx < self.nx as isize && ny >= 0 && ny < self.ny as isize {
                    let idx = ny as usize * self.nx + nx as usize;
                    for &u in &self.cells[idx] {
                        if u != v {
                            result.push(u);
                        }
                    }
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cycle(n: u32) -> Graph {
        let mut g = Graph::with_vertices(n);
        for i in 0..n {
            g.add_edge(i, (i + 1) % n).unwrap();
        }
        g
    }

    fn make_path(n: u32) -> Graph {
        let mut g = Graph::with_vertices(n);
        for i in 0..n - 1 {
            g.add_edge(i, i + 1).unwrap();
        }
        g
    }

    #[test]
    fn empty_graph() {
        let g = Graph::with_vertices(0);
        let coords = layout_fruchterman_reingold(&g, &FrParams::default()).unwrap();
        assert!(coords.is_empty());
    }

    #[test]
    fn single_vertex() {
        let g = Graph::with_vertices(1);
        let coords = layout_fruchterman_reingold(&g, &FrParams::default()).unwrap();
        assert_eq!(coords.len(), 1);
    }

    #[test]
    fn cycle5_produces_finite_coords() {
        let g = make_cycle(5);
        let coords = layout_fruchterman_reingold(&g, &FrParams::default()).unwrap();
        assert_eq!(coords.len(), 5);
        for &(x, y) in &coords {
            assert!(x.is_finite());
            assert!(y.is_finite());
        }
    }

    #[test]
    fn disconnected_graph_doesnt_panic() {
        let mut g = Graph::with_vertices(6);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();
        g.add_edge(4, 5).unwrap();
        let coords = layout_fruchterman_reingold(&g, &FrParams::default()).unwrap();
        assert_eq!(coords.len(), 6);
        for &(x, y) in &coords {
            assert!(x.is_finite());
            assert!(y.is_finite());
        }
    }

    #[test]
    fn grid_mode_works() {
        let g = make_path(10);
        let params = FrParams {
            grid: FrGrid::Grid,
            niter: 50,
            ..Default::default()
        };
        let coords = layout_fruchterman_reingold(&g, &params).unwrap();
        assert_eq!(coords.len(), 10);
        for &(x, y) in &coords {
            assert!(x.is_finite());
            assert!(y.is_finite());
        }
    }

    #[test]
    fn weighted_edges() {
        let g = make_cycle(4);
        let params = FrParams {
            weights: Some(vec![1.0, 2.0, 1.0, 2.0]),
            niter: 100,
            ..Default::default()
        };
        let coords = layout_fruchterman_reingold(&g, &params).unwrap();
        assert_eq!(coords.len(), 4);
    }

    #[test]
    fn invalid_weights_rejected() {
        let g = make_cycle(4);
        let params = FrParams {
            weights: Some(vec![1.0, -1.0, 1.0, 1.0]),
            ..Default::default()
        };
        assert!(layout_fruchterman_reingold(&g, &params).is_err());
    }

    #[test]
    fn bounds_constrain_output() {
        let g = make_path(5);
        let params = FrParams {
            niter: 200,
            bounds: FrBounds {
                minx: Some(vec![0.0; 5]),
                maxx: Some(vec![1.0; 5]),
                miny: Some(vec![0.0; 5]),
                maxy: Some(vec![1.0; 5]),
            },
            ..Default::default()
        };
        let coords = layout_fruchterman_reingold(&g, &params).unwrap();
        for &(x, y) in &coords {
            assert!((0.0..=1.0).contains(&x), "x={x} out of bounds");
            assert!((0.0..=1.0).contains(&y), "y={y} out of bounds");
        }
    }

    #[test]
    fn layout_3d_basic() {
        let g = make_cycle(5);
        let coords = layout_fruchterman_reingold_3d(&g, &FrParams3d::default()).unwrap();
        assert_eq!(coords.len(), 5);
        for &(x, y, z) in &coords {
            assert!(x.is_finite());
            assert!(y.is_finite());
            assert!(z.is_finite());
        }
    }

    #[test]
    fn seed_coords_used() {
        let g = make_path(3);
        let seed = vec![(0.0, 0.0), (1.0, 0.0), (2.0, 0.0)];
        let params = FrParams {
            niter: 0,
            seed_coords: Some(seed.clone()),
            ..Default::default()
        };
        let coords = layout_fruchterman_reingold(&g, &params).unwrap();
        assert_eq!(coords, seed);
    }
}
