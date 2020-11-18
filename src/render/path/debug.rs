//! Draw path, lines and so on for debug purposes.

use crate::config::GameEngineConfig;
use crate::core::colors::RgbaColor;
use crate::render::path::{Color, Position, Vertex};
use crate::resources::Resources;
use lyon::math::Point;
use lyon::tessellation::geometry_builder::simple_builder;
use lyon::tessellation::{basic_shapes, StrokeOptions, VertexBuffers};

pub struct DebugQueue(Vec<(Vec<Vertex>, Vec<u16>)>);

impl Default for DebugQueue {
    fn default() -> Self {
        Self(vec![])
    }
}

impl DebugQueue {
    pub fn drain(&mut self) -> std::vec::Drain<(Vec<Vertex>, Vec<u16>)> {
        self.0.drain(..)
    }
}

fn show_gizmo(resources: &Resources) -> bool {
    resources
        .fetch::<GameEngineConfig>()
        .map(|conf| conf.show_gizmos)
        .unwrap_or(false)
}

pub fn stroke_circle(resources: &Resources, position: glam::Vec2, radius: f32, color: RgbaColor) {
    if !show_gizmo(resources) {
        return;
    }

    match resources.fetch_mut::<DebugQueue>() {
        Some(mut debug_queue) => {
            let mut geometry: VertexBuffers<Point, u16> = VertexBuffers::new();
            let color = color.to_normalized();
            if let Err(e) = basic_shapes::stroke_circle(
                Point::new(position.x(), position.y()),
                radius,
                &StrokeOptions::default(),
                &mut simple_builder(&mut geometry),
            ) {
                error!("Error during stroke_line = {:?}", e);
                return;
            }

            debug_queue.0.push((
                geometry
                    .vertices
                    .iter()
                    .map(|p| Vertex {
                        position: Position::new([p.x, p.y]),
                        color: Color::new(color),
                    })
                    .collect::<Vec<_>>(),
                geometry.indices,
            ));
        }
        None => error!("No DebugQueue in resources"),
    }
}

pub fn stroke_quad(
    resources: &Resources,
    position: glam::Vec2,
    dimensions: glam::Vec2,
    color: RgbaColor,
) {
    if !show_gizmo(resources) {
        return;
    }

    match resources.fetch_mut::<DebugQueue>() {
        Some(mut debug_queue) => {
            let mut geometry: VertexBuffers<Point, u16> = VertexBuffers::new();
            let color = color.to_normalized();

            let p1 = position;
            let p2 = position + glam::Vec2::unit_x() * dimensions.x();
            let p3 = position
                + glam::Vec2::unit_x() * dimensions.x()
                + glam::Vec2::unit_y() * dimensions.y();
            let p4 = position + glam::Vec2::unit_y() * dimensions.y();
            if let Err(e) = basic_shapes::stroke_quad(
                Point::new(p1.x(), p1.y()),
                Point::new(p2.x(), p2.y()),
                Point::new(p3.x(), p3.y()),
                Point::new(p4.x(), p4.y()),
                &StrokeOptions::default(),
                &mut simple_builder(&mut geometry),
            ) {
                error!("Error during stroke_line = {:?}", e);
                return;
            }

            debug_queue.0.push((
                geometry
                    .vertices
                    .iter()
                    .map(|p| Vertex {
                        position: Position::new([p.x, p.y]),
                        color: Color::new(color),
                    })
                    .collect::<Vec<_>>(),
                geometry.indices,
            ));
        }
        None => error!("No DebugQueue in resources"),
    }
}

pub fn stroke_line(
    resources: &Resources,
    position: glam::Vec2,
    target: glam::Vec2,
    color: RgbaColor,
) {
    if !show_gizmo(resources) {
        return;
    }

    match resources.fetch_mut::<DebugQueue>() {
        Some(mut debug_queue) => {
            let mut geometry: VertexBuffers<Point, u16> = VertexBuffers::new();
            let color = color.to_normalized();
            if let Err(e) = basic_shapes::stroke_polyline(
                vec![
                    Point::new(position.x(), position.y()),
                    Point::new(target.x(), target.y()),
                ],
                false,
                &StrokeOptions::default(),
                &mut simple_builder(&mut geometry),
            ) {
                error!("Error during stroke_line = {:?}", e);
                return;
            }

            debug_queue.0.push((
                geometry
                    .vertices
                    .iter()
                    .map(|p| Vertex {
                        position: Position::new([p.x, p.y]),
                        color: Color::new(color),
                    })
                    .collect::<Vec<_>>(),
                geometry.indices,
            ));
        }
        None => error!("No DebugQueue in resources"),
    }
}
