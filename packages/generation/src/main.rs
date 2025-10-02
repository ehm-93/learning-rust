use generation::freeform;
use rayon::prelude::*;

pub fn main() {
    (0..10).into_par_iter().for_each(|i| {
        freeform::freeform(
            &format!("out/output_{}.png", i),
            &format!("out/flooded_{}.png", i),
        );
    });
}
