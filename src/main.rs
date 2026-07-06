
mod brep;
mod camera;
mod render_options;

use brep::{BRepGraph, Point3};
use camera::Camera;
use minifb::{Key, Window, WindowOptions};
use render_options::{Color, ProjectionKind, RenderOptions};

fn main() {
    let mut graph = BRepGraph::new();
    let v0 = graph.add_vertex(Point3::new(-0.5, -0.5, -0.5));
    let v1 = graph.add_vertex(Point3::new(0.5, -0.5, -0.5));
    let v2 = graph.add_vertex(Point3::new(0.5, 0.5, -0.5));
    let v3 = graph.add_vertex(Point3::new(-0.5, 0.5, -0.5));
    let v4 = graph.add_vertex(Point3::new(-0.5, -0.5, 0.5));
    let v5 = graph.add_vertex(Point3::new(0.5, -0.5, 0.5));
    let v6 = graph.add_vertex(Point3::new(0.5, 0.5, 0.5));
    let v7 = graph.add_vertex(Point3::new(-0.5, 0.5, 0.5));

    let _ = graph.add_edge(v0, v1);
    let _ = graph.add_edge(v1, v2);
    let _ = graph.add_edge(v2, v3);
    let _ = graph.add_edge(v3, v0);
    let _ = graph.add_edge(v4, v5);
    let _ = graph.add_edge(v5, v6);
    let _ = graph.add_edge(v6, v7);
    let _ = graph.add_edge(v7, v4);
    let _ = graph.add_edge(v0, v4);
    let _ = graph.add_edge(v1, v5);
    let _ = graph.add_edge(v2, v6);
    let _ = graph.add_edge(v3, v7);

    let width = 800;
    let height = 600;
    let mut buffer = vec![0_u32; width * height];
    let mut render_options = RenderOptions::default();
    let camera = Camera::default();
    render_options.set_projection_kind(ProjectionKind::Perspective);
    render_options.set_perspective_fov(0.0);
    render_options.set_background_color(Color::new(0x3a, 0x3a, 0x3a));
    render_options.set_edge_color(Color::new(0xff, 0xff, 0xff));
    render_options.set_vertex_color(Color::new(0x49, 0x95, 0xdd));

    let mut window = Window::new(
        "forgecad - BRep preview",
        width,
        height,
        WindowOptions::default(),
    )
    .expect("Unable to create window");

    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));

    let angle = 0.0f32;
    let mut p_is_down = false;
    let mut o_is_down = false;
    let mut a_is_down = false;
    let mut s_is_down = false;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let p_down = window.is_key_down(Key::P);
        let o_down = window.is_key_down(Key::O);
        let a_down = window.is_key_down(Key::A);
        let s_down = window.is_key_down(Key::S);

        if p_down && !p_is_down {
            render_options.set_projection_kind(ProjectionKind::Perspective);
        }
        if o_down && !o_is_down {
            render_options.set_projection_kind(ProjectionKind::Orthographic);
        }
        if a_down && !a_is_down {
            render_options.set_perspective_fov(render_options.perspective_fov + 5.0);
        }
        if s_down && !s_is_down {
            render_options.set_perspective_fov(render_options.perspective_fov - 5.0);
        }

        p_is_down = p_down;
        o_is_down = o_down;
        a_is_down = a_down;
        s_is_down = s_down;

        buffer.iter_mut().for_each(|pixel| *pixel = render_options.background_color.to_u32());
        render_graph(&mut buffer, width, height, &graph, angle, &render_options, &camera);
        window.update_with_buffer(&buffer, width, height).unwrap();
    }
}

fn render_graph(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    graph: &BRepGraph,
    angle: f32,
    render_options: &RenderOptions,
    camera: &Camera,
) {
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let ortho_scale = height as f32 / 2.0;
    let fov = render_options.perspective_fov.max(0.001);
    let perspective_scale = (height as f32 / 2.0) / (fov.to_radians() / 2.0).tan();

    for edge in graph.edges() {
        let Some((from, to)) = edge.endpoints() else {
            continue;
        };
        let p0 = rotate_z(graph.vertex_position(from), angle);
        let p1 = rotate_z(graph.vertex_position(to), angle);

        if let (Some(start), Some(end)) = (
            project_point(p0, center_x, center_y, ortho_scale, perspective_scale, render_options, camera),
            project_point(p1, center_x, center_y, ortho_scale, perspective_scale, render_options, camera),
        ) {
            draw_line(buffer, width, height, start, end, render_options.edge_color.to_u32());
        }
    }

    let axis_length = 0.25;
    let axis_lines = [
        (Point3::new(0.0, 0.0, 0.0), Point3::new(axis_length, 0.0, 0.0), 0xFF0000_u32),
        (Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, axis_length, 0.0), 0x00FF00_u32),
        (Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, axis_length), 0x0000FF_u32),
    ];

    for (from, to, color) in axis_lines {
        if let (Some(start), Some(end)) = (
            project_point(from, center_x, center_y, ortho_scale, perspective_scale, render_options, camera),
            project_point(to, center_x, center_y, ortho_scale, perspective_scale, render_options, camera),
        ) {
            draw_line(buffer, width, height, start, end, color);
        }
    }

    for point in graph.vertices() {
        let rotated = rotate_z(point, angle);
        if let Some(projected) = project_point(rotated, center_x, center_y, ortho_scale, perspective_scale, render_options, camera) {
            draw_point(buffer, width, height, projected, render_options.vertex_color.to_u32());
        }
    }

    if let Some(origin) = project_point(Point3::new(0.0, 0.0, 0.0), center_x, center_y, ortho_scale, perspective_scale, render_options, camera) {
        draw_point(buffer, width, height, origin, 0xFF0000_u32);
    }
}

fn project_point(
    point: Point3,
    center_x: f32,
    center_y: f32,
    ortho_scale: f32,
    perspective_scale: f32,
    render_options: &RenderOptions,
    camera: &Camera,
) -> Option<(i32, i32)> {
    let view = camera.view_matrix();
    let world = point;

    let view_x = view[0] * world.x + view[4] * world.y + view[8] * world.z + view[12];
    let view_y = view[1] * world.x + view[5] * world.y + view[9] * world.z + view[13];
    let view_z = view[2] * world.x + view[6] * world.y + view[10] * world.z + view[14];

    if view_z >= -0.01 {
        return None;
    }

    let ortho_x = center_x + view_x * ortho_scale;
    let ortho_y = center_y - view_y * ortho_scale;

    let screen_x = center_x + (view_x * perspective_scale) / (-view_z);
    let screen_y = center_y - (view_y * perspective_scale) / (-view_z);

    let screen = match render_options.projection_kind {
        ProjectionKind::Orthographic => (ortho_x, ortho_y),
        ProjectionKind::Perspective => {
            let blend = (render_options.perspective_fov / 90.0).clamp(0.0, 1.0);
            if blend <= 0.0 {
                (ortho_x, ortho_y)
            } else {
                (
                    ortho_x * (1.0 - blend) + screen_x * blend,
                    ortho_y * (1.0 - blend) + screen_y * blend,
                )
            }
        }
    };

    Some((screen.0 as i32, screen.1 as i32))
}

fn rotate_z(point: Point3, angle: f32) -> Point3 {
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    Point3::new(
        point.x * cos_a - point.y * sin_a,
        point.x * sin_a + point.y * cos_a,
        point.z,
    )
}

fn draw_point(buffer: &mut [u32], width: usize, height: usize, point: (i32, i32), color: u32) {
    let (cx, cy) = point;
    let radius = 2;
    let max_x = (width as i32).saturating_sub(1);
    let max_y = (height as i32).saturating_sub(1);

    for y in -radius..=radius {
        for x in -radius..=radius {
            if x * x + y * y > radius * radius {
                continue;
            }

            let px = cx.saturating_add(x).clamp(0, max_x);
            let py = cy.saturating_add(y).clamp(0, max_y);
            let idx = (py as usize) * width + (px as usize);
            if let Some(pixel) = buffer.get_mut(idx) {
                *pixel = color;
            }
        }
    }
}

fn draw_line(buffer: &mut [u32], width: usize, height: usize, start: (i32, i32), end: (i32, i32), color: u32) {
    let max_x = (width as i32).saturating_sub(1);
    let max_y = (height as i32).saturating_sub(1);
    let (x0, y0) = start;
    let (x1, y1) = end;
    let x0 = x0.clamp(0, max_x);
    let y0 = y0.clamp(0, max_y);
    let x1 = x1.clamp(0, max_x);
    let y1 = y1.clamp(0, max_y);
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = x0;
    let mut y = y0;

    loop {
        let idx = (y as usize) * width + (x as usize);
        if let Some(pixel) = buffer.get_mut(idx) {
            *pixel = color;
        }
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x = x.saturating_add(sx);
        }
        if e2 <= dx {
            err += dx;
            y = y.saturating_add(sy);
        }
    }
}
