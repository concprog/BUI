use std::collections::VecDeque;

pub(crate) type Position = [f32; 2];

const DIAMETER: f32 = 100.;
const LAYER_GAP: f32 = 60.;
const NODE_GAP: f32 = 40.;
const SWEEPS: usize = 12;
const ALIGN_PASSES: usize = 8;

const LINK_DISTANCE: f64 = 150.;
const LINK_STRENGTH: f64 = 0.5;
const FORCE_X: f64 = 2000.;
const FORCE_X_STRENGTH: f64 = 0.02;
const COLLIDE: f64 = 100.;
const TICKS: usize = 300;
const VELOCITY_DECAY: f64 = 0.6;
const JIGGLE: f64 = 1e-6;

const GRID_VERTICAL: f32 = 140.;
const GRID_HORIZONTAL: f32 = 120.;

fn center(positions: &mut [Position]) {
    if positions.is_empty() {
        return;
    }
    let (mut left, mut right, mut top, mut bottom) = (
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::INFINITY,
        f32::NEG_INFINITY,
    );
    for [x, y] in positions.iter() {
        left = left.min(*x);
        right = right.max(*x);
        top = top.min(*y);
        bottom = bottom.max(*y);
    }
    let (cx, cy) = (f32::midpoint(left, right), f32::midpoint(top, bottom));
    for [x, y] in positions.iter_mut() {
        *x -= cx;
        *y -= cy;
    }
}

fn acyclic(count: usize, edges: &[(usize, usize)]) -> Vec<(usize, usize)> {
    let mut outgoing: Vec<Vec<usize>> = vec![Vec::new(); count];
    for (index, (source, target)) in edges.iter().enumerate() {
        if source != target {
            outgoing[*source].push(index);
        }
    }
    let mut state = vec![0u8; count];
    let mut reversed = vec![false; edges.len()];
    for start in 0..count {
        if state[start] != 0 {
            continue;
        }
        let mut stack: Vec<(usize, usize)> = vec![(start, 0)];
        state[start] = 1;
        while let Some((node, next)) = stack.last_mut() {
            if let Some(edge) = outgoing[*node].get(*next).copied() {
                *next += 1;
                let target = edges[edge].1;
                match state[target] {
                    0 => {
                        state[target] = 1;
                        stack.push((target, 0));
                    }
                    1 => reversed[edge] = true,
                    _ => {}
                }
            } else {
                state[*node] = 2;
                stack.pop();
            }
        }
    }
    edges
        .iter()
        .zip(reversed)
        .filter(|((source, target), _)| source != target)
        .map(|(&(source, target), flip)| {
            if flip {
                (target, source)
            } else {
                (source, target)
            }
        })
        .collect()
}

fn layers(count: usize, edges: &[(usize, usize)]) -> Vec<usize> {
    let mut incoming = vec![0usize; count];
    let mut outgoing: Vec<Vec<usize>> = vec![Vec::new(); count];
    for &(source, target) in edges {
        incoming[target] += 1;
        outgoing[source].push(target);
    }
    let mut layer = vec![0usize; count];
    let mut queue: VecDeque<usize> = (0..count).filter(|node| incoming[*node] == 0).collect();
    while let Some(node) = queue.pop_front() {
        for &target in &outgoing[node] {
            layer[target] = layer[target].max(layer[node] + 1);
            incoming[target] -= 1;
            if incoming[target] == 0 {
                queue.push_back(target);
            }
        }
    }
    layer
}

pub(crate) fn layered(count: usize, edges: &[(usize, usize)]) -> Vec<Position> {
    if count == 0 {
        return Vec::new();
    }
    let edges = acyclic(count, edges);
    let mut layer = layers(count, &edges);
    let mut links: Vec<(usize, usize)> = Vec::with_capacity(edges.len());
    for &(source, target) in &edges {
        let mut from = source;
        for step in layer[source] + 1..layer[target] {
            let dummy = layer.len();
            layer.push(step);
            links.push((from, dummy));
            from = dummy;
        }
        links.push((from, target));
    }
    let total = layer.len();
    let depth = layer.iter().copied().max().unwrap_or(0) + 1;
    let mut up: Vec<Vec<usize>> = vec![Vec::new(); total];
    let mut down: Vec<Vec<usize>> = vec![Vec::new(); total];
    for &(source, target) in &links {
        down[source].push(target);
        up[target].push(source);
    }
    let mut rows: Vec<Vec<usize>> = vec![Vec::new(); depth];
    for (node, at) in layer.iter().enumerate() {
        rows[*at].push(node);
    }
    let mut order = vec![0f32; total];
    let renumber = |rows: &[Vec<usize>], order: &mut [f32]| {
        for row in rows {
            for (index, node) in row.iter().enumerate() {
                order[*node] = index as f32;
            }
        }
    };
    renumber(&rows, &mut order);
    for sweep in 0..SWEEPS {
        let downward = sweep % 2 == 0;
        let range: Vec<usize> = if downward {
            (1..depth).collect()
        } else {
            (0..depth.saturating_sub(1)).rev().collect()
        };
        for at in range {
            let neighbours = if downward { &up } else { &down };
            let mut keyed: Vec<(f32, usize)> = rows[at]
                .iter()
                .map(|node| {
                    let around = &neighbours[*node];
                    let key = if around.is_empty() {
                        order[*node]
                    } else {
                        around.iter().map(|other| order[*other]).sum::<f32>() / around.len() as f32
                    };
                    (key, *node)
                })
                .collect();
            keyed.sort_by(|a, b| a.0.total_cmp(&b.0));
            rows[at] = keyed.into_iter().map(|(_, node)| node).collect();
            for (index, node) in rows[at].iter().enumerate() {
                order[*node] = index as f32;
            }
        }
    }
    let step = DIAMETER + NODE_GAP;
    let mut y = vec![0f32; total];
    for row in &rows {
        let offset = (row.len() as f32 - 1.) * step / 2.;
        for (index, node) in row.iter().enumerate() {
            y[*node] = index as f32 * step - offset;
        }
    }
    for pass in 0..ALIGN_PASSES {
        let downward = pass % 2 == 0;
        for at in 0..depth {
            let at = if downward { at } else { depth - 1 - at };
            let row = &rows[at];
            if row.is_empty() {
                continue;
            }
            let wanted: Vec<f32> = row
                .iter()
                .map(|node| {
                    let around: Vec<usize> =
                        up[*node].iter().chain(&down[*node]).copied().collect();
                    if around.is_empty() {
                        y[*node]
                    } else {
                        around.iter().map(|other| y[*other]).sum::<f32>() / around.len() as f32
                    }
                })
                .collect();
            let mut placed = wanted.clone();
            for index in 1..placed.len() {
                placed[index] = placed[index].max(placed[index - 1] + step);
            }
            let drift =
                (wanted.iter().sum::<f32>() - placed.iter().sum::<f32>()) / placed.len() as f32;
            for (node, at) in row.iter().zip(placed) {
                y[*node] = at + drift;
            }
        }
    }
    let mut positions: Vec<Position> = (0..count)
        .map(|node| [layer[node] as f32 * (DIAMETER + LAYER_GAP), y[node]])
        .collect();
    center(&mut positions);
    positions
}

struct Body {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    fixed: Option<f64>,
}

#[allow(clippy::too_many_lines)]
pub(crate) fn force(count: usize, edges: &[(usize, usize)]) -> Vec<Position> {
    if count == 0 {
        return Vec::new();
    }
    let mut incoming = vec![0usize; count];
    let mut outgoing: Vec<Vec<usize>> = vec![Vec::new(); count];
    let mut degree = vec![0usize; count];
    for &(source, target) in edges {
        incoming[target] += 1;
        outgoing[source].push(target);
        degree[source] += 1;
        degree[target] += 1;
    }
    let mut roots: Vec<usize> = (0..count).filter(|node| incoming[*node] == 0).collect();
    if roots.is_empty() {
        roots.push(0);
    }
    let mut placed: Vec<Option<(f64, f64)>> = vec![None; count];
    let mut root_y = 0.;
    for &root in &roots {
        let mut level = vec![root];
        let mut x = 0.;
        while !level.is_empty() {
            let mut next = Vec::new();
            let mut y = root_y;
            for node in level {
                if placed[node].is_some() {
                    continue;
                }
                placed[node] = Some((x, y));
                y += 200.;
                next.extend(outgoing[node].iter().copied());
            }
            level = next;
            x += 200.;
        }
        root_y += 300.;
    }
    let golden = std::f64::consts::PI * (3. - 5f64.sqrt());
    let mut bodies: Vec<Body> = placed
        .iter()
        .enumerate()
        .map(|(index, at)| {
            let (x, y) = at.unwrap_or_else(|| {
                let radius = 10. * (0.5 + index as f64).sqrt();
                let angle = index as f64 * golden;
                (radius * angle.cos(), radius * angle.sin())
            });
            Body {
                x,
                y,
                vx: 0.,
                vy: 0.,
                fixed: None,
            }
        })
        .collect();
    for &root in &roots {
        bodies[root].fixed = Some(bodies[root].x);
        for &child in &outgoing[root] {
            bodies[child].fixed = Some(bodies[child].x);
        }
    }
    let links: Vec<(usize, usize, f64)> = edges
        .iter()
        .filter(|(source, target)| source != target)
        .map(|&(source, target)| {
            let bias = degree[source] as f64 / (degree[source] + degree[target]) as f64;
            (source, target, bias)
        })
        .collect();
    let decay = 1. - 0.001f64.powf(1. / TICKS as f64);
    let mut alpha = 1.;
    let cell = COLLIDE * 2.;
    let mut buckets: std::collections::HashMap<(i64, i64), Vec<usize>> =
        std::collections::HashMap::new();
    for _ in 0..TICKS {
        alpha += -alpha * decay;
        for &(source, target, bias) in &links {
            let (s, t) = (&bodies[source], &bodies[target]);
            let mut x = t.x + t.vx - s.x - s.vx;
            let mut y = t.y + t.vy - s.y - s.vy;
            if x == 0. {
                x = JIGGLE;
            }
            if y == 0. {
                y = JIGGLE;
            }
            let length = x.hypot(y);
            let scale = (length - LINK_DISTANCE) / length * alpha * LINK_STRENGTH;
            let (x, y) = (x * scale, y * scale);
            bodies[target].vx -= x * bias;
            bodies[target].vy -= y * bias;
            bodies[source].vx += x * (1. - bias);
            bodies[source].vy += y * (1. - bias);
        }
        for body in &mut bodies {
            body.vx += (FORCE_X - body.x) * FORCE_X_STRENGTH * alpha;
        }
        buckets.values_mut().for_each(Vec::clear);
        let key = |x: f64, y: f64| ((x / cell).floor() as i64, (y / cell).floor() as i64);
        for (index, body) in bodies.iter().enumerate() {
            buckets
                .entry(key(body.x + body.vx, body.y + body.vy))
                .or_default()
                .push(index);
        }
        let reach = COLLIDE * 2.;
        for index in 0..count {
            let (xi, yi) = (
                bodies[index].x + bodies[index].vx,
                bodies[index].y + bodies[index].vy,
            );
            let (cx, cy) = key(xi, yi);
            for dx in -1..=1 {
                for dy in -1..=1 {
                    let Some(others) = buckets.get(&(cx + dx, cy + dy)) else {
                        continue;
                    };
                    for &other in others {
                        if other <= index {
                            continue;
                        }
                        let o = &bodies[other];
                        let mut x = xi - o.x - o.vx;
                        let mut y = yi - o.y - o.vy;
                        let mut length = x * x + y * y;
                        if length >= reach * reach {
                            continue;
                        }
                        if x == 0. {
                            x = JIGGLE;
                            length += x * x;
                        }
                        if y == 0. {
                            y = JIGGLE;
                            length += y * y;
                        }
                        let distance = length.sqrt();
                        let push = (reach - distance) / distance;
                        let (x, y) = (x * push * 0.5, y * push * 0.5);
                        bodies[index].vx += x;
                        bodies[index].vy += y;
                        bodies[other].vx -= x;
                        bodies[other].vy -= y;
                    }
                }
            }
        }
        for body in &mut bodies {
            if let Some(fixed) = body.fixed {
                body.x = fixed;
                body.vx = 0.;
            } else {
                body.vx *= VELOCITY_DECAY;
                body.x += body.vx;
            }
            body.vy *= VELOCITY_DECAY;
            body.y += body.vy;
        }
    }
    let mut positions: Vec<Position> = bodies
        .iter()
        .map(|body| [body.x as f32, body.y as f32])
        .collect();
    center(&mut positions);
    positions
}

pub(crate) fn grid(count: usize, width: f32) -> Vec<Position> {
    let padding = GRID_HORIZONTAL / 2.;
    let per_row =
        (((width - padding * 2.) / GRID_VERTICAL).floor().max(1.) as usize).min(count.max(1));
    let middle = ((per_row - 1) as f32 * GRID_HORIZONTAL / 2.).floor();
    (0..count)
        .map(|index| {
            let (row, column) = (index / per_row, index % per_row);
            [
                column as f32 * GRID_HORIZONTAL - middle,
                row as f32 * GRID_VERTICAL - 60.,
            ]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layered_puts_each_hop_in_the_next_layer() {
        let positions = layered(4, &[(0, 1), (1, 2), (0, 3), (3, 2)]);
        assert!(positions[0][0] < positions[1][0]);
        assert!(positions[1][0] < positions[2][0]);
        assert!((positions[1][0] - positions[3][0]).abs() < 1e-3);
        assert!((positions[1][1] - positions[3][1]).abs() >= DIAMETER + NODE_GAP - 1e-3);
    }

    #[test]
    fn layered_survives_cycles() {
        let positions = layered(3, &[(0, 1), (1, 2), (2, 0)]);
        assert_eq!(positions.len(), 3);
        assert!(
            positions
                .iter()
                .all(|[x, y]| x.is_finite() && y.is_finite())
        );
    }

    #[test]
    fn force_separates_nodes() {
        let positions = force(5, &[(0, 1), (0, 2), (0, 3), (0, 4)]);
        for a in 0..5 {
            for b in a + 1..5 {
                let (dx, dy) = (
                    positions[a][0] - positions[b][0],
                    positions[a][1] - positions[b][1],
                );
                assert!(dx.hypot(dy) > 50.);
            }
        }
    }

    #[test]
    fn grid_wraps_rows() {
        let positions = grid(4, 400.);
        assert!((positions[0][1] - positions[1][1]).abs() < 1e-3);
        assert!(positions[3][1] > positions[0][1]);
    }
}
