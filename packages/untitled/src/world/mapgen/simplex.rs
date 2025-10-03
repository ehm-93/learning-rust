use noise::{NoiseFn, OpenSimplex};
use rand::Rng;

/// Generates simplex noise and applies it to the map based on the given scale and threshold.
///
/// # Parameters
/// - `map`: A mutable reference to a 2D vector representing the map where noise will be applied.
/// - `rng`: A mutable reference to a random number generator implementing the `Rng`
/// - `scale`: A scaling factor to adjust the frequency of the noise.
/// - `threshold`: A (0, 1) threshold value to determine which noise values will set map cells to true
pub fn generate_simplex_noise(map: &mut Vec<Vec<bool>>, rng: &mut impl Rng, scale: f64, threshold: f64) {
    let width = map[0].len();
    let height = map.len();
    let noise = OpenSimplex::new(rng.random());

    for y in 0..height {
        for x in 0..width {
            let nx = x as f64 * scale;
            let ny = y as f64 * scale;
            let value = noise.get([nx, ny]) + 0.5; // Normalize to [0, 1]
            if value > threshold {
                map[y][x] = true;
            }
        }
    }
}
