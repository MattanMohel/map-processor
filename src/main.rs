pub mod draw;
pub mod reader;
use image::{GenericImageView, RgbaImage};
use reader::Reader;

fn main() {
    // let mut reader = Reader::new(1, "src/assets/Floor 1");
    // reader.build_data();
    // let mut reader = Reader::new(2, "src/assets/Floor 2");
    // reader.build_data();
    // let mut reader = Reader::new(3, "src/assets/Floor 3");
    // reader.build_data();

    draw::run_sketch();
}
