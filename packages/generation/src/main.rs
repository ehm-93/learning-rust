use generation::freeform;
use rayon::prelude::*;

pub fn main() {
    (0..16).into_par_iter().for_each(|i| {
        freeform::freeform(
            256 * (i + 1),
            &format!("out/output_{}.png", i),
            &format!("out/flooded_{}.png", i),
        );
    });
}
