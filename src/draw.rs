use std::collections::HashMap;

use nannou::{
    image::{DynamicImage, ImageBuffer},
    prelude::*,
};

use crate::reader::Reader;

const TEXT_SIZE_PX: f32 = 50.;
const POINT_RADIUS: i32 = 15;
const VISITED_RADIUS: i32 = 8;
type RbgaBuffer = ImageBuffer<nannou::image::Rgba<u8>, Vec<u8>>;

pub struct Model {
    texture: Option<wgpu::Texture>,
    background: wgpu::Texture,
	visited: HashMap<usize, String>,
    buffer: String,
    reader: Reader,
    index: i32,
    trailing_index: i32,
	shift_pressed: bool,
}

pub fn run_sketch() {
    nannou::app(model).update(update).run();
}

pub fn model(app: &App) -> Model {
    app.new_window()
        .title("Label")
        .size(400, 400)
        .resizable(true)
        .view(view)
        .key_pressed(key_pressed)
        .key_released(key_released)
        .build()
        .unwrap();

    let mut reader = Reader::new(1);
    reader.build_data();

    let background_image = reader.background_image();
    let background = wgpu::Texture::from_image(app, background_image);

    Model {
        texture: None,
        background,
		visited: HashMap::new(),
        buffer: String::new(),
        reader,
        index: 0,
        trailing_index: -1,
		shift_pressed: false,
    }
}

fn draw_point_at(buffer: &mut RbgaBuffer, (pos_x, pos_y): (i32, i32), color: [u8; 4], radius: i32) {
	for x in pos_x - radius..=pos_x + radius {
		for y in pos_y - radius..=pos_y + radius {
			if f32::hypot(x as f32 - pos_x as f32, y as f32 - pos_y as f32) < POINT_RADIUS as f32 {
				*buffer.get_pixel_mut(x as u32, y as u32) = nannou::image::Rgba::from(color);
			}
		}
	}
}

pub fn update(app: &App, model: &mut Model, _: Update) {
	if app.mouse.buttons.left().is_down() {
        let (point_x, point_y) = app.main_window().inner_size_points();
		let (pixel_x, _) = app.main_window().inner_size_pixels();
        let [image_x, image_y] = model.background.size();
        let x = app.mouse.x + point_x as f32 / 2.;
        let y = point_y as f32 / 2. - app.mouse.y;

		if x >= 0. && x <= point_x && y >= 0. && y <= point_y - TEXT_SIZE_PX * point_x / pixel_x as f32 {
			let x_in_pixels = x * image_x as f32 / point_x;
			let y_in_pixels = y * image_y as f32 / (point_y - TEXT_SIZE_PX * point_x / pixel_x as f32); 

			for (i, point) in model.reader.points().iter().enumerate() {
				let (point_x, point_y) = point.position();
				if f32::hypot(x_in_pixels as f32 - point_x as f32, y_in_pixels as f32 - point_y as f32) < POINT_RADIUS as f32 {
					model.index = i as i32;
					if let Some(name) = model.visited.get(&i) {
						model.buffer = name.clone();
					} else {
						model.buffer.clear();
					}
					break;
				}
			}
		} 
    }

    if model.trailing_index != model.index {
        let mut buffer = ImageBuffer::new(model.reader.width() as u32, model.reader.height() as u32);
        for (px1, px2) in model.reader.composite_image().pixels().zip(buffer.pixels_mut()) {
            *px2 = *px1;
        }

        let (point_x, point_y) = model.reader.points()[model.index as usize].position();
		draw_point_at(&mut buffer, (point_x, point_y), [255, 0, 0, 255], POINT_RADIUS);
		for i in model.visited.keys() {
			let (pos_x, pos_y) = model.reader.points()[*i].position();
			draw_point_at(&mut buffer, (pos_x, pos_y), [0, 255, 0, 255], VISITED_RADIUS);
		}

        let image = DynamicImage::ImageRgba8(buffer);
        let texture = wgpu::Texture::from_image(app, &image);
        model.texture = Some(texture);
        model.trailing_index = model.index
    }
}

pub fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let (x, y) = app.main_window().inner_size_points();
    let (px, _) = app.main_window().inner_size_pixels();

    draw.background().color(WHITESMOKE);
    draw.texture(&model.background)
        .w_h(x, y - TEXT_SIZE_PX)
        .x_y(0., TEXT_SIZE_PX / 2.);

    if let Some(texture) = &model.texture {
        draw.texture(texture)
            .w_h(x, y - TEXT_SIZE_PX)
            .x_y(0., TEXT_SIZE_PX / 2.);
    }

    let rect = Rect::from_w_h(px as f32, TEXT_SIZE_PX * x as f32 / px as f32)
        .mid_bottom_of(app.window_rect());

    draw.rect().wh(rect.wh()).xy(rect.xy()).color(WHITE);
    draw.text(&model.buffer)
        .color(BLACK)
        .x_y(0., (y - TEXT_SIZE_PX) / -2.)
        .font_size(20);

    draw.to_frame(app, &frame).unwrap();
}

pub fn key_pressed(_: &App, model: &mut Model, key: Key) {
    key as usize;

    if key as usize <= Key::Z as usize && key as usize >= Key::A as usize {
        let ch = ('a' as u8 + key as u8 - Key::A as u8) as char;
		if model.shift_pressed {
			let cap = (ch as u8 - 32) as char;
			model.buffer.push(cap);
		} else {
			model.buffer.push(ch);
		}
    } else {
        match key {
            Key::Back => {
                model.buffer.pop();
            }
            Key::Space => {
                model.buffer.push(' ');
            }
			Key::Escape | Key::Tab => {
            	model.reader.save_to_file();
			}
            Key::Return => {
				if model.buffer.is_empty() {
					return;
				}

				if let Some(name) = model.visited.get(&(model.index as usize)) {
					model.reader.json_replace(&name, &model.buffer);
				} else {
					let name = model.reader.points()[model.index as usize].hash();
					model.reader.json_replace(&name, &model.buffer);
				}

				model.visited.insert(model.index as usize, model.buffer.clone());
                model.buffer.clear();
                model.index = (model.index + 1) % model.reader.points().len() as i32;
            }
			Key::RShift | Key::LShift => model.shift_pressed = true,
            Key::Apostrophe => model.buffer.push('\''),
			Key::Backslash => model.buffer = model.reader.points()[model.index as usize].hash(),
            Key::Key0 => model.buffer.push('0'),
            Key::Key1 => model.buffer.push('1'),
            Key::Key2 => model.buffer.push('2'),
            Key::Key3 => model.buffer.push('3'),
            Key::Key4 => model.buffer.push('4'),
            Key::Key5 => model.buffer.push('5'),
            Key::Key6 => model.buffer.push('6'),
            Key::Key7 => model.buffer.push('7'),
            Key::Key8 => model.buffer.push('8'),
            Key::Key9 => model.buffer.push('9'),
            _ => (),
        }
    }
}

pub fn key_released(_: &App, model: &mut Model, key: Key) {
	match key {
		Key::RShift | Key::LShift => model.shift_pressed = false,
		_ => (),
    }
}