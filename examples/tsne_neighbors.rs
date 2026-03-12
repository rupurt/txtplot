use colored::Color;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use txtplot::ChartContext;

const DIM: usize = 6;
const CLUSTERS: usize = 4;
const POINTS_PER_CLUSTER: usize = 12;
const PERPLEXITY: f64 = 8.0;
const ITERATIONS: usize = 300;
const K_NEIGHBORS: usize = 3;
const EPSILON: f64 = 1e-12;
const PLOT_LEFT: usize = 6;
const PLOT_RIGHT: usize = 6;
const PLOT_TOP: usize = 9;
const PLOT_BOTTOM: usize = 4;

#[derive(Clone, Copy, Debug, Default)]
struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy)]
struct Sample {
    label: usize,
    features: [f64; DIM],
}

fn gaussian(rng: &mut StdRng) -> f64 {
    let u1 = rng.gen::<f64>().clamp(f64::MIN_POSITIVE, 1.0);
    let u2 = rng.gen::<f64>();
    (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}

fn generate_samples() -> Vec<Sample> {
    let centers = [
        [-2.4, -1.8, 0.4, 1.1, -0.9, 0.3],
        [2.2, -1.4, 1.5, -0.7, 0.8, -1.2],
        [-1.2, 2.4, -1.8, 1.0, 0.4, 1.7],
        [2.5, 1.6, 0.1, -1.5, -1.1, 0.9],
    ];
    let mut rng = StdRng::seed_from_u64(42);
    let mut samples = Vec::with_capacity(CLUSTERS * POINTS_PER_CLUSTER);

    for (label, center) in centers.iter().enumerate() {
        for index in 0..POINTS_PER_CLUSTER {
            let phase = index as f64 / POINTS_PER_CLUSTER as f64 * std::f64::consts::TAU;
            let mut features = [0.0; DIM];

            for dim in 0..DIM {
                let curved_signal = match dim {
                    0 => 0.35 * phase.cos(),
                    1 => 0.25 * phase.sin(),
                    2 => 0.18 * (2.0 * phase).sin(),
                    3 => 0.22 * (phase + label as f64 * 0.4).cos(),
                    4 => 0.15 * (2.0 * phase + 0.3).cos(),
                    _ => 0.20 * (phase - 0.5).sin(),
                };
                features[dim] = center[dim] + curved_signal + gaussian(&mut rng) * 0.18;
            }

            samples.push(Sample { label, features });
        }
    }

    samples
}

fn squared_distance(a: &[f64; DIM], b: &[f64; DIM]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(lhs, rhs)| {
            let delta = lhs - rhs;
            delta * delta
        })
        .sum()
}

fn pairwise_squared_distances(samples: &[Sample]) -> Vec<f64> {
    let count = samples.len();
    let mut distances = vec![0.0; count * count];

    for i in 0..count {
        for j in (i + 1)..count {
            let distance = squared_distance(&samples[i].features, &samples[j].features);
            distances[i * count + j] = distance;
            distances[j * count + i] = distance;
        }
    }

    distances
}

fn entropy_for_row(distances: &[f64], count: usize, row: usize, beta: f64) -> (f64, Vec<f64>) {
    let mut probabilities = vec![0.0; count];
    let mut sum = 0.0;

    for col in 0..count {
        if col == row {
            continue;
        }
        let weight = (-distances[row * count + col] * beta).exp();
        probabilities[col] = weight;
        sum += weight;
    }

    if sum <= EPSILON {
        return (0.0, probabilities);
    }

    let mut entropy = 0.0;
    for probability in &mut probabilities {
        if *probability <= 0.0 {
            continue;
        }
        *probability /= sum;
        entropy -= *probability * probability.ln();
    }

    (entropy, probabilities)
}

fn joint_probabilities(distances: &[f64], perplexity: f64, count: usize) -> Vec<f64> {
    let target_entropy = perplexity.ln();
    let mut conditional = vec![0.0; count * count];

    for row in 0..count {
        let mut beta = 1.0;
        let mut beta_min = f64::NEG_INFINITY;
        let mut beta_max = f64::INFINITY;
        let mut best = vec![0.0; count];

        for _ in 0..60 {
            let (entropy, probabilities) = entropy_for_row(distances, count, row, beta);
            best = probabilities;
            let diff = entropy - target_entropy;

            if diff.abs() < 1e-5 {
                break;
            }

            if diff > 0.0 {
                beta_min = beta;
                beta = if beta_max.is_finite() {
                    0.5 * (beta + beta_max)
                } else {
                    beta * 2.0
                };
            } else {
                beta_max = beta;
                beta = if beta_min.is_finite() {
                    0.5 * (beta + beta_min)
                } else {
                    beta * 0.5
                };
            }
        }

        conditional[row * count..(row + 1) * count].copy_from_slice(&best);
    }

    let mut joint = vec![0.0; count * count];
    let normalizer = 2.0 * count as f64;

    for i in 0..count {
        for j in (i + 1)..count {
            let value = (conditional[i * count + j] + conditional[j * count + i]) / normalizer;
            joint[i * count + j] = value;
            joint[j * count + i] = value;
        }
    }

    let sum: f64 = joint.iter().sum();
    if sum > EPSILON {
        for value in &mut joint {
            *value /= sum;
        }
    }

    joint
}

fn initial_embedding(samples: &[Sample]) -> Vec<Vec2> {
    let count = samples.len();
    let mean_x = samples.iter().map(|sample| sample.features[0]).sum::<f64>() / count as f64;
    let mean_y = samples.iter().map(|sample| sample.features[1]).sum::<f64>() / count as f64;

    let mut embedding: Vec<Vec2> = samples
        .iter()
        .map(|sample| Vec2::new(sample.features[0] - mean_x, sample.features[1] - mean_y))
        .collect();

    let scale = embedding
        .iter()
        .map(|point| point.x.abs().max(point.y.abs()))
        .fold(0.0, f64::max)
        .max(1.0);

    for point in &mut embedding {
        point.x /= scale;
        point.y /= scale;
    }

    embedding
}

fn tsne_embedding(samples: &[Sample], distances: &[f64]) -> Vec<Vec2> {
    let count = samples.len();
    let probabilities = joint_probabilities(distances, PERPLEXITY, count);
    let mut embedding = initial_embedding(samples);
    let mut velocity = vec![Vec2::default(); count];

    for iteration in 0..ITERATIONS {
        let exaggeration = if iteration < 90 { 8.0 } else { 1.0 };
        let momentum = if iteration < 90 { 0.45 } else { 0.8 };
        let learning_rate = if iteration < 90 { 40.0 } else { 60.0 };

        let mut numerators = vec![0.0; count * count];
        let mut q_sum = 0.0;

        for i in 0..count {
            for j in (i + 1)..count {
                let dx = embedding[i].x - embedding[j].x;
                let dy = embedding[i].y - embedding[j].y;
                let numerator = 1.0 / (1.0 + dx * dx + dy * dy);
                numerators[i * count + j] = numerator;
                numerators[j * count + i] = numerator;
                q_sum += 2.0 * numerator;
            }
        }

        let inv_q_sum = 1.0 / q_sum.max(EPSILON);
        let mut gradients = vec![Vec2::default(); count];

        for i in 0..count {
            for j in (i + 1)..count {
                let dx = embedding[i].x - embedding[j].x;
                let dy = embedding[i].y - embedding[j].y;
                let numerator = numerators[i * count + j];
                let q = numerator * inv_q_sum;
                let p = probabilities[i * count + j] * exaggeration;
                let force = 4.0 * (p - q) * numerator;

                gradients[i].x += force * dx;
                gradients[i].y += force * dy;
                gradients[j].x -= force * dx;
                gradients[j].y -= force * dy;
            }
        }

        for index in 0..count {
            let grad_norm = (gradients[index].x * gradients[index].x
                + gradients[index].y * gradients[index].y)
                .sqrt();
            let clip = if grad_norm > 5.0 {
                5.0 / grad_norm
            } else {
                1.0
            };

            velocity[index].x =
                momentum * velocity[index].x - learning_rate * gradients[index].x * clip;
            velocity[index].y =
                momentum * velocity[index].y - learning_rate * gradients[index].y * clip;
            embedding[index].x += velocity[index].x;
            embedding[index].y += velocity[index].y;
        }

        let mean_x = embedding.iter().map(|point| point.x).sum::<f64>() / count as f64;
        let mean_y = embedding.iter().map(|point| point.y).sum::<f64>() / count as f64;
        for point in &mut embedding {
            point.x -= mean_x;
            point.y -= mean_y;
        }
    }

    let scale = embedding
        .iter()
        .map(|point| point.x.abs().max(point.y.abs()))
        .fold(0.0, f64::max)
        .max(1.0);
    for point in &mut embedding {
        point.x /= scale;
        point.y /= scale;
    }

    embedding
}

fn knn_graph(distances: &[f64], count: usize, k: usize) -> Vec<(usize, usize)> {
    let mut edges = Vec::new();

    for row in 0..count {
        let mut neighbors: Vec<(f64, usize)> = (0..count)
            .filter(|&col| col != row)
            .map(|col| (distances[row * count + col], col))
            .collect();
        neighbors.sort_by(|left, right| left.0.total_cmp(&right.0));

        for (_, col) in neighbors.into_iter().take(k) {
            let edge = if row < col { (row, col) } else { (col, row) };
            if !edges.contains(&edge) {
                edges.push(edge);
            }
        }
    }

    edges
}

fn map_point(
    chart: &ChartContext,
    point: Vec2,
    x_range: (f64, f64),
    y_range: (f64, f64),
) -> (isize, isize) {
    let width_px = chart.canvas.pixel_width();
    let height_px = chart.canvas.pixel_height();
    let x_span = (x_range.1 - x_range.0).max(EPSILON);
    let y_span = (y_range.1 - y_range.0).max(EPSILON);
    let drawable_width = (width_px.saturating_sub(PLOT_LEFT + PLOT_RIGHT + 1)).max(1) as f64;
    let drawable_height = (height_px.saturating_sub(PLOT_TOP + PLOT_BOTTOM + 1)).max(1) as f64;

    let x_norm = ((point.x - x_range.0) / x_span).clamp(0.0, 1.0);
    let y_norm = ((point.y - y_range.0) / y_span).clamp(0.0, 1.0);
    let px = PLOT_LEFT as f64 + x_norm * drawable_width;
    let py = PLOT_TOP as f64 + (1.0 - y_norm) * drawable_height;

    (px as isize, py as isize)
}

fn stamp_point(chart: &mut ChartContext, x: isize, y: isize, color: Color) {
    for dx in -1..=1 {
        for dy in -1..=1 {
            let px = x + dx;
            let py = y + dy;
            if px < 0 || py < 0 {
                continue;
            }
            let px = px as usize;
            let py = py as usize;
            if px >= chart.canvas.pixel_width() || py >= chart.canvas.pixel_height() {
                continue;
            }
            chart.canvas.set_pixel_screen(px, py, Some(color));
        }
    }
}

fn main() {
    let samples = generate_samples();
    let distances = pairwise_squared_distances(&samples);
    let edges = knn_graph(&distances, samples.len(), K_NEIGHBORS);
    let embedding = tsne_embedding(&samples, &distances);

    let colors = [
        Color::BrightCyan,
        Color::BrightYellow,
        Color::BrightMagenta,
        Color::BrightGreen,
    ];

    let points: Vec<(f64, f64)> = embedding.iter().map(|point| (point.x, point.y)).collect();
    let (x_range, y_range) = ChartContext::get_auto_range(&points, 0.10);
    let mut chart = ChartContext::new(82, 24);

    for &(from, to) in &edges {
        let start = map_point(&chart, embedding[from], x_range, y_range);
        let end = map_point(&chart, embedding[to], x_range, y_range);
        chart
            .canvas
            .line_screen(start.0, start.1, end.0, end.1, Some(Color::BrightBlack));
    }

    for (index, point) in embedding.iter().enumerate() {
        let (px, py) = map_point(&chart, *point, x_range, y_range);
        stamp_point(&mut chart, px, py, colors[samples[index].label]);
    }

    chart.text(
        "t-SNE on synthetic 6D clusters",
        0.10,
        0.96,
        Some(Color::White),
    );
    chart.text(
        "gray edges: 3-NN graph | colors: source clusters",
        0.10,
        0.90,
        Some(Color::BrightBlack),
    );

    println!(
        "{}",
        chart
            .canvas
            .render_with_options(true, Some("t-SNE + nearest-neighbor graph"))
    );
}
