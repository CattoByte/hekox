use nannou::prelude::*;

fn main() {
    nannou::app(model).run();
}

struct Model {
    window_id: window::Id,
    texture: wgpu::Texture,
}

fn model(app: &App) -> Model {
    let window_id = app.new_window().size(512, 512).view(view).build().unwrap();

    // Load the image from disk and upload it to a GPU texture.
    let assets = app.assets_path().unwrap();
    let img_path = assets.join("../assets/").join("test.png");
    let texture = wgpu::Texture::from_path(app, img_path).unwrap();

    Model { window_id, texture }
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(DIMGRAY);

    let window = app.window(model.window_id).unwrap();
    let win_rect = window.rect();
    let draw = app.draw();

    let centre = pt3(0.0, 0.0, 0.0);
    let size = vec3(1.0, 1.0, 1.0);
    let cuboid = geom::Cuboid::from_xyz_whd(centre, size);
    let points = cuboid
        .triangles_iter()
        .flat_map(geom::Tri::vertices)
        .map(|point| {
            let [x, y, _] = point;
            let tex_coords = [x + 0.5, 1.0 - (y + 0.5)];
            (point, tex_coords)
        });

    let cube_side = win_rect.w().min(win_rect.h()) * 0.5;
    draw.text("Hekox")
        .color(WHITE)
        .font_size((60 + (app.time.sin() * 20.0) as i32) as u32);
    draw.scale(cube_side)
        .mesh()
        .points_textured(&model.texture, points)
        .z_radians(app.time.sin() * 3.0)
        .x_radians(app.time.cos() * 3.0)
        .y_radians((app.time / 4.0).tan());

    draw.to_frame(app, &frame).unwrap();
}
