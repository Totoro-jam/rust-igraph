//! Layout merging via Diffusion Limited Aggregation (ALGO-LO-014).
//!
//! Merges multiple 2D component layouts into a single combined layout
//! by covering each with a bounding circle and placing them using DLA
//! random walks so they do not overlap.
//!
//! Reference: igraph C `src/layout/merge_dla.c` + `merge_grid.c`.

use crate::core::IgraphResult;

/// Merge multiple 2D layouts into a single combined layout using DLA placement.
///
/// Each component layout is covered by a bounding circle. The largest
/// component is placed at the origin, then subsequent components are
/// placed using a DLA (Diffusion Limited Aggregation) random walk that
/// finds a position touching an already-placed component.
///
/// # Arguments
///
/// * `layouts` — slice of component layouts, each a `Vec<[f64; 2]>`.
///
/// Returns a single merged layout containing all vertices from all
/// components in order.
///
/// # Examples
///
/// ```
/// use rust_igraph::layout_merge_dla;
///
/// let layout1 = vec![[0.0, 0.0], [1.0, 0.0], [0.5, 0.5]];
/// let layout2 = vec![[0.0, 0.0], [1.0, 1.0]];
/// let merged = layout_merge_dla(&[&layout1, &layout2]).unwrap();
/// assert_eq!(merged.len(), 5);
/// assert!(merged.iter().all(|p| p[0].is_finite() && p[1].is_finite()));
/// ```
pub fn layout_merge_dla(layouts: &[&[[f64; 2]]]) -> IgraphResult<Vec<[f64; 2]>> {
    let n_components = layouts.len();

    if n_components == 0 {
        return Ok(Vec::new());
    }
    if n_components == 1 {
        return Ok(layouts[0].to_vec());
    }

    // Compute bounding spheres for each component
    let mut centers_x = vec![0.0_f64; n_components];
    let mut centers_y = vec![0.0_f64; n_components];
    let mut radii = vec![0.0_f64; n_components];
    let mut placement_radii = vec![0.0_f64; n_components];
    let mut total_nodes = 0usize;

    let mut area = 0.0_f64;
    for (i, layout) in layouts.iter().enumerate() {
        let size = layout.len();
        total_nodes += size;

        if size == 0 {
            continue;
        }

        // Compute bounding sphere
        let (cx, cy, r) = bounding_circle(layout);
        centers_x[i] = cx;
        centers_y[i] = cy;
        radii[i] = r;

        // Placement radius scales with component size
        placement_radii[i] = (size as f64).powf(0.75);
        area += placement_radii[i] * placement_radii[i];
    }

    // Sort components by size (largest first)
    let mut order: Vec<usize> = (0..n_components).collect();
    order.sort_unstable_by(|&a, &b| layouts[b].len().cmp(&layouts[a].len()));

    // Create merge grid
    let grid_extent = (5.0 * area).sqrt();
    let mut grid = MergeGrid::new(
        -grid_extent,
        grid_extent,
        200,
        -grid_extent,
        grid_extent,
        200,
    );

    // Place components by DLA
    let mut placed_x = vec![0.0_f64; n_components];
    let mut placed_y = vec![0.0_f64; n_components];

    // Place the largest at origin
    let first = order[0];
    placed_x[first] = 0.0;
    placed_y[first] = 0.0;
    grid.place_sphere(0.0, 0.0, placement_radii[first], first);

    let mut rng = SplitMix64::new(42);
    let startr = grid_extent;
    let killr = grid_extent + 5.0;

    // Place remaining via DLA random walk
    for &comp_idx in order.iter().skip(1) {
        let r = placement_radii[comp_idx];
        if r <= 0.0 {
            placed_x[comp_idx] = 0.0;
            placed_y[comp_idx] = 0.0;
            continue;
        }

        let (px, py) = dla_walk(&grid, r, 0.0, 0.0, startr, killr, &mut rng);
        placed_x[comp_idx] = px;
        placed_y[comp_idx] = py;
        grid.place_sphere(px, py, r, comp_idx);
    }

    // Build the result: transform each component's vertices
    let mut result = Vec::with_capacity(total_nodes);
    for (i, layout) in layouts.iter().enumerate() {
        let ox = placed_x[i];
        let oy = placed_y[i];
        let nr = radii[i];
        let scale = if nr > 0.0 {
            placement_radii[i] / nr
        } else {
            1.0
        };
        let cx = centers_x[i];
        let cy = centers_y[i];

        for p in *layout {
            result.push([scale * (p[0] - cx) + ox, scale * (p[1] - cy) + oy]);
        }
    }

    Ok(result)
}

fn bounding_circle(points: &[[f64; 2]]) -> (f64, f64, f64) {
    if points.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    if points.len() == 1 {
        return (points[0][0], points[0][1], 0.0);
    }

    let mut xmin = points[0][0];
    let mut xmax = points[0][0];
    let mut ymin = points[0][1];
    let mut ymax = points[0][1];

    for p in points.iter().skip(1) {
        if p[0] < xmin {
            xmin = p[0];
        }
        if p[0] > xmax {
            xmax = p[0];
        }
        if p[1] < ymin {
            ymin = p[1];
        }
        if p[1] > ymax {
            ymax = p[1];
        }
    }

    let cx = (xmin + xmax) / 2.0;
    let cy = (ymin + ymax) / 2.0;
    let dx = xmax - xmin;
    let dy = ymax - ymin;
    let r = (dx * dx + dy * dy).sqrt() / 2.0;

    (cx, cy, r)
}

fn dla_walk(
    grid: &MergeGrid,
    r: f64,
    cx: f64,
    cy: f64,
    startr: f64,
    killr: f64,
    rng: &mut SplitMix64,
) -> (f64, f64) {
    let max_attempts = 100_000;

    for _ in 0..max_attempts {
        // Start particle at random position on circle of radius startr
        let angle = rng.next_uniform() * std::f64::consts::TAU;
        let len = rng.next_uniform() * 0.5 * startr + 0.5 * startr;
        let mut x = cx + len * angle.cos();
        let mut y = cy + len * angle.sin();

        // Check if starting position already collides
        if grid.get_sphere(x, y, r) >= 0 {
            continue;
        }

        // Random walk until collision or kill
        let step_size = startr / 100.0;
        loop {
            let dist = ((x - cx) * (x - cx) + (y - cy) * (y - cy)).sqrt();
            if dist >= killr {
                break;
            }

            let a = rng.next_uniform() * std::f64::consts::TAU;
            let l = rng.next_uniform() * step_size;
            let nx = x + l * a.cos();
            let ny = y + l * a.sin();

            if grid.get_sphere(nx, ny, r) >= 0 {
                return (x, y);
            }
            x = nx;
            y = ny;
        }
    }

    // Fallback: place at startr away from center
    let angle = rng.next_uniform() * std::f64::consts::TAU;
    (cx + startr * angle.cos(), cy + startr * angle.sin())
}

// ═══════════════════════════════════════════════════════════════════
// Merge Grid
// ═══════════════════════════════════════════════════════════════════

struct MergeGrid {
    minx: f64,
    maxx: f64,
    miny: f64,
    maxy: f64,
    stepsx: usize,
    stepsy: usize,
    deltax: f64,
    deltay: f64,
    data: Vec<i32>, // 0 = empty, id+1 = occupied by component id
}

impl MergeGrid {
    fn new(minx: f64, maxx: f64, stepsx: usize, miny: f64, maxy: f64, stepsy: usize) -> Self {
        Self {
            minx,
            maxx,
            miny,
            maxy,
            stepsx,
            stepsy,
            deltax: (maxx - minx) / stepsx as f64,
            deltay: (maxy - miny) / stepsy as f64,
            data: vec![0; stepsx * stepsy],
        }
    }

    fn which(&self, xc: f64, yc: f64) -> (usize, usize) {
        let cx = if xc <= self.minx {
            0
        } else if xc >= self.maxx {
            self.stepsx - 1
        } else {
            ((xc - self.minx) / self.deltax) as usize
        };

        let cy = if yc <= self.miny {
            0
        } else if yc >= self.maxy {
            self.stepsy - 1
        } else {
            ((yc - self.miny) / self.deltay) as usize
        };

        (cx.min(self.stepsx - 1), cy.min(self.stepsy - 1))
    }

    fn place_sphere(&mut self, x: f64, y: f64, r: f64, id: usize) {
        let (cx, cy) = self.which(x, y);
        let val = (id as i32) + 1;

        self.data[cy * self.stepsx + cx] = val;

        // Fill all four quadrants within radius
        // Quadrant 1: +x, +y
        let mut i = 0i32;
        while (cx as i32 + i) < self.stepsx as i32 {
            let gx = self.minx + (cx as f64 + i as f64) * self.deltax;
            if Self::grid_dist(x, y, gx, y) >= r {
                break;
            }
            let mut j = 0i32;
            while (cy as i32 + j) < self.stepsy as i32 {
                let gy = self.miny + (cy as f64 + j as f64) * self.deltay;
                if Self::grid_dist(x, y, gx, gy) >= r {
                    break;
                }
                let idx = (cy as i32 + j) as usize * self.stepsx + (cx as i32 + i) as usize;
                self.data[idx] = val;
                j += 1;
            }
            i += 1;
        }

        // Quadrant 2: +x, -y
        i = 0;
        while (cx as i32 + i) < self.stepsx as i32 {
            let gx = self.minx + (cx as f64 + i as f64) * self.deltax;
            if Self::grid_dist(x, y, gx, y) >= r {
                break;
            }
            let mut j = 1i32;
            while (cy as i32 - j) > 0 {
                let gy = self.miny + (cy as f64 - j as f64 + 1.0) * self.deltay;
                if Self::grid_dist(x, y, gx, gy) >= r {
                    break;
                }
                let idx = (cy as i32 - j) as usize * self.stepsx + (cx as i32 + i) as usize;
                self.data[idx] = val;
                j += 1;
            }
            i += 1;
        }

        // Quadrant 3: -x, +y
        i = 1;
        while (cx as i32 - i) > 0 {
            let gx = self.minx + (cx as f64 - i as f64 + 1.0) * self.deltax;
            if Self::grid_dist(x, y, gx, y) >= r {
                break;
            }
            let mut j = 0i32;
            while (cy as i32 + j) < self.stepsy as i32 {
                let gy = self.miny + (cy as f64 + j as f64) * self.deltay;
                if Self::grid_dist(x, y, gx, gy) >= r {
                    break;
                }
                let idx = (cy as i32 + j) as usize * self.stepsx + (cx as i32 - i) as usize;
                self.data[idx] = val;
                j += 1;
            }
            i += 1;
        }

        // Quadrant 4: -x, -y
        i = 1;
        while (cx as i32 - i) > 0 {
            let gx = self.minx + (cx as f64 - i as f64 + 1.0) * self.deltax;
            if Self::grid_dist(x, y, gx, y) >= r {
                break;
            }
            let mut j = 1i32;
            while (cy as i32 - j) > 0 {
                let gy = self.miny + (cy as f64 - j as f64 + 1.0) * self.deltay;
                if Self::grid_dist(x, y, gx, gy) >= r {
                    break;
                }
                let idx = (cy as i32 - j) as usize * self.stepsx + (cx as i32 - i) as usize;
                self.data[idx] = val;
                j += 1;
            }
            i += 1;
        }
    }

    fn get_sphere(&self, x: f64, y: f64, r: f64) -> i32 {
        if x - r <= self.minx || x + r >= self.maxx || y - r <= self.miny || y + r >= self.maxy {
            return -1;
        }

        let (cx, cy) = self.which(x, y);
        let ret = self.data[cy * self.stepsx + cx] - 1;
        if ret >= 0 {
            return ret;
        }

        // Check four quadrants
        if let Some(id) = self.check_quadrant(x, y, r, cx, cy, 1, 1) {
            return id;
        }
        if let Some(id) = self.check_quadrant(x, y, r, cx, cy, 1, -1) {
            return id;
        }
        if let Some(id) = self.check_quadrant(x, y, r, cx, cy, -1, 1) {
            return id;
        }
        if let Some(id) = self.check_quadrant(x, y, r, cx, cy, -1, -1) {
            return id;
        }

        -1
    }

    fn check_quadrant(
        &self,
        x: f64,
        y: f64,
        r: f64,
        cx: usize,
        cy: usize,
        dx: i32,
        dy: i32,
    ) -> Option<i32> {
        let i_start: i32 = i32::from(dx <= 0);
        let j_start: i32 = i32::from(dy <= 0);

        let mut i = i_start;
        loop {
            let gxi = cx as i32 + i * dx;
            if gxi < 0 || gxi >= self.stepsx as i32 {
                break;
            }
            let gx = self.minx + gxi as f64 * self.deltax;
            if Self::grid_dist(x, y, gx, y) >= r {
                break;
            }

            let mut j = j_start;
            loop {
                let gyj = cy as i32 + j * dy;
                if gyj < 0 || gyj >= self.stepsy as i32 {
                    break;
                }
                let gy = self.miny + gyj as f64 * self.deltay;
                if Self::grid_dist(x, y, gx, gy) >= r {
                    break;
                }

                let idx = gyj as usize * self.stepsx + gxi as usize;
                let val = self.data[idx] - 1;
                if val >= 0 {
                    return Some(val);
                }
                j += 1;
            }
            i += 1;
        }
        None
    }

    fn grid_dist(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
        let dx = x1 - x2;
        let dy = y1 - y2;
        (dx * dx + dy * dy).sqrt()
    }
}

// ═══════════════════════════════════════════════════════════════════
// Internal RNG
// ═══════════════════════════════════════════════════════════════════

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn next_uniform(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / ((1u64 << 53) as f64)
    }
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_empty() {
        let result = layout_merge_dla(&[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_merge_single() {
        let layout = vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let result = layout_merge_dla(&[&layout]).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_merge_two_components() {
        let l1 = vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let l2 = vec![[0.0, 0.0], [1.0, 1.0]];
        let result = layout_merge_dla(&[&l1, &l2]).unwrap();
        assert_eq!(result.len(), 5);
        for p in &result {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }

    #[test]
    fn test_merge_three_components() {
        let l1 = vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0]];
        let l2 = vec![[0.0, 0.0], [1.0, 1.0], [2.0, 2.0]];
        let l3 = vec![[0.0, 0.0], [0.5, 0.5]];
        let result = layout_merge_dla(&[&l1, &l2, &l3]).unwrap();
        assert_eq!(result.len(), 9);
        for p in &result {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }

    #[test]
    fn test_merge_single_vertex_components() {
        let l1 = vec![[0.0, 0.0]];
        let l2 = vec![[1.0, 1.0]];
        let l3 = vec![[2.0, 2.0]];
        let result = layout_merge_dla(&[&l1, &l2, &l3]).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_merge_empty_component() {
        let l1: Vec<[f64; 2]> = vec![];
        let l2 = vec![[0.0, 0.0], [1.0, 0.0]];
        let result = layout_merge_dla(&[&l1[..], &l2]).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_merge_deterministic() {
        let l1 = vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let l2 = vec![[0.0, 0.0], [1.0, 1.0]];
        let r1 = layout_merge_dla(&[&l1, &l2]).unwrap();
        let r2 = layout_merge_dla(&[&l1, &l2]).unwrap();
        for i in 0..5 {
            assert!((r1[i][0] - r2[i][0]).abs() < 1e-10);
            assert!((r1[i][1] - r2[i][1]).abs() < 1e-10);
        }
    }

    #[test]
    fn test_merge_no_overlap() {
        let l1 = vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0], [0.5, -1.0]];
        let l2 = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]];
        let result = layout_merge_dla(&[&l1, &l2]).unwrap();
        // Component centers should be separated
        let cx1 = (result[0][0] + result[1][0] + result[2][0] + result[3][0]) / 4.0;
        let cy1 = (result[0][1] + result[1][1] + result[2][1] + result[3][1]) / 4.0;
        let cx2 = (result[4][0] + result[5][0] + result[6][0]) / 3.0;
        let cy2 = (result[4][1] + result[5][1] + result[6][1]) / 3.0;
        let dist = ((cx1 - cx2) * (cx1 - cx2) + (cy1 - cy2) * (cy1 - cy2)).sqrt();
        assert!(dist > 0.1);
    }
}
