use noise::{utils::*, Perlin, Seedable};

fn main() {
    let perlin = Perlin::new();

    PlaneMapBuilder::new(&perlin)
        .set_size(1024, 1024)
        .set_x_bounds(-15.0, 15.0)
        .set_y_bounds(-15.0, 15.0)
        .build()
        .write_to_file("perlin.png");

    let perlin = perlin.set_seed(1);

    PlaneMapBuilder::new(&perlin)
        .set_size(1024, 1024)
        .set_x_bounds(-15.0, 15.0)
        .set_y_bounds(-15.0, 15.0)
        .build()
        .write_to_file("perlin_seed=1.png");
}
