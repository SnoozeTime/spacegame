use lyon::lyon_tessellation::geometry_builder::simple_builder;
use lyon::lyon_tessellation::{BuffersBuilder, StrokeAttributes, StrokeOptions};
use lyon::math::Point;
use lyon::tessellation::{basic_shapes, VertexBuffers};

pub struct MyVertex([f32; 2]);
fn main() {
    let mut geometry: VertexBuffers<Point, u16> = VertexBuffers::new();
    // let mut bb = BuffersBuilder::new(&mut geometry, |pos: Point, _: StrokeAttributes| {
    //     MyVertex(pos.to_array())
    // });
    basic_shapes::stroke_circle(
        Point::new(0.0, 0.0),
        100.0,
        &StrokeOptions::default(),
        &mut simple_builder(&mut geometry),
    );

    println!("{:#?}", geometry.vertices);
}
