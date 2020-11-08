use rand::prelude::StdRng;
use rand::SeedableRng;

#[derive(Debug, Copy, Clone)]
pub struct Seed(pub(crate) [u8; 32]);

pub struct RandomGenerator {
    rand: StdRng,
}

impl RandomGenerator {
    pub fn new(seed: Seed) -> Self {
        Self {
            rand: StdRng::from_seed(seed.0),
        }
    }

    pub fn from_entropy() -> Self {
        Self {
            rand: StdRng::from_entropy(),
        }
    }

    pub fn rng(&mut self) -> &mut StdRng {
        &mut self.rand
    }
}
