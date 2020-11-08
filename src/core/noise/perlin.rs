use rand::{seq::SliceRandom, SeedableRng};

///  6t5-15t4+10t3.
fn fade(t: f32) -> f32 {
    t * t * t * (10.0 + t * (6.0 * t - 15.0))
}

pub fn make_permutation<R: SeedableRng + rand::RngCore>(rand: &mut R) -> Vec<u8> {
    let mut p: Vec<u8> = (0..255u8).collect();
    p.shuffle(rand);
    for i in 0..p.len() {
        p.push(p[i]);
    }
    p
}

fn get_constant_vector(hash: usize) -> glam::Vec2 {
    let h = hash & 3;

    match h {
        0 => glam::Vec2::one(),
        1 => glam::Vec2::new(-1.0, 1.0),
        2 => glam::Vec2::new(-1.0, -1.0),
        _ => glam::Vec2::new(1.0, -1.0),
    }
}
pub struct PermutationTable(Vec<u8>);

impl PermutationTable {
    fn get1(&self, x: isize) -> usize {
        let idx = (x & 0xFF) as usize;
        self.0[idx] as usize
    }

    fn get2(&self, x: isize, y: isize) -> usize {
        let y = (y & 0xFF) as usize;
        self.0[self.get1(x) ^ y] as usize
    }
}

pub fn perlin2d(x: f32, y: f32, perm: &PermutationTable) -> f32 {
    let xf = x - x.floor();
    let yf = y - y.floor();

    let near_corner = [x.floor() as isize, y.floor() as isize];
    let far_corner = [near_corner[0] + 1, near_corner[1] + 1];
    let near_distance = [x - x.floor(), y - y.floor()];
    let _far_distance = [near_distance[0] - 1.0, near_distance[1] - 1.0];

    let top_right: glam::Vec2 = glam::vec2(xf - 1.0, yf - 1.0);
    let top_left: glam::Vec2 = glam::vec2(xf, yf - 1.0);
    let bottom_right: glam::Vec2 = glam::vec2(xf - 1.0, yf);
    let bottom_left: glam::Vec2 = glam::vec2(xf, yf);

    // select a value in the array for each of the corner.
    let value_top_right = perm.get2(far_corner[0], far_corner[1]);
    let value_top_left = perm.get2(near_corner[0], far_corner[1]);
    let value_bottom_right = perm.get2(far_corner[0], near_corner[1]);
    let value_bottom_left = perm.get2(near_corner[0], near_corner[1]); //perm[(perm[X] + Y) & 0xFF];

    let dot_top_right = top_right.dot(get_constant_vector(value_top_right));
    let dot_top_left = top_left.dot(get_constant_vector(value_top_left));
    let dot_bottom_right = bottom_right.dot(get_constant_vector(value_bottom_right));

    // near corner
    let dot_bottom_left = bottom_left.dot(get_constant_vector(value_bottom_left));

    let u = fade(near_distance[0]);
    let v = fade(near_distance[1]);

    let res = bilinear_interpolation(
        u,
        v,
        dot_bottom_left,
        dot_top_left,
        dot_bottom_right,
        dot_top_right,
    ) * 2.0
        * (2.0_f32).sqrt();
    clamp((res + 1.0) / 2., 0.0, 1.0)
}

fn clamp(x: f32, min: f32, max: f32) -> f32 {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}

#[inline(always)]
fn bilinear_interpolation(u: f32, v: f32, g00: f32, g01: f32, g10: f32, g11: f32) -> f32 {
    let k0 = g00;
    let k1 = g10 - g00;
    let k2 = g01 - g00;
    let k3 = g00 + g11 - g10 - g01;
    k0 + k1 * u + k2 * v + k3 * u * v
}

pub struct Perlin {
    perm: PermutationTable,
    repeat: Option<usize>,
}

impl Perlin {
    pub fn new<R: SeedableRng + rand::RngCore>(rand: &mut R) -> Self {
        Self {
            perm: PermutationTable(make_permutation(rand)),
            repeat: None,
        }
    }

    pub fn with_repeat(mut self, repeat: usize) -> Self {
        self.repeat = Some(repeat);
        self
    }

    pub fn perlin(&self, x: f32, y: f32) -> f32 {
        perlin2d(x, y, &self.perm)
    }

    pub fn octave_perlin(&self, x: f32, y: f32, octaves: u32, persistence: f32) -> f32 {
        let mut freq = 1.0;
        let mut amplitude = 1.0;
        let mut max_value = 0.0;
        let mut total = 0.0;
        for _ in 0..octaves {
            total += perlin2d(x * freq, y * freq, &self.perm) * amplitude;

            max_value += amplitude;
            amplitude *= persistence;
            freq *= 2.0;
        }

        total / max_value
    }
}
