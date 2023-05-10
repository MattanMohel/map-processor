use image::GenericImageView;
use json::{array, object, JsonValue};
use nannou::image::{DynamicImage, Rgba, RgbaImage};

const POINT_COLOR: [u8; 4] = [1, 0, 0, 255];
const JOINT_COLOR: [u8; 4] = [2, 0, 0, 255];
type Point = (i32, i32);

#[derive(Clone)]
pub struct Region {
    pub cells: Vec<Point>,
    pub nodes: Vec<String>,
}

impl Region {
    pub fn new(cells: Vec<Point>) -> Self {
        Self {
            cells,
            nodes: Vec::new(),
        }
    }

    pub fn position(&self) -> Point {
        let mut x_avg = 0;
        let mut y_avg = 0;

        for (x, y) in self.cells.iter() {
            x_avg += *x;
            y_avg += *y;
        }

        (
            x_avg / self.cells.len() as i32,
            y_avg / self.cells.len() as i32,
        )
    }

    pub fn hash(&self) -> String {
        let (x, y) = self.position();
        format!("{}{}", x, y)
    }
}

pub struct Reader {
    floor: usize,
    composite: RgbaImage,
    background: DynamicImage,
    visited: Vec<Vec<bool>>,
    regions: Vec<Region>,
    points: Vec<Region>,
    width: usize,
    height: usize,
    json_data: String,
    save_directory: String,
}

impl Reader {
    pub fn new(floor: usize) -> Self {
        let point_path = format!("src/assets/Floor {floor}/points.PNG");
        let joint_path = format!("src/assets/Floor {floor}/joints.PNG");
        let backround_path = format!("src/assets/Floor {floor}/bg.PNG");
        let save_path = format!("src/assets/Floor {floor}/floor-{floor}.json");

        let points = image::io::Reader::open(point_path)
            .expect("points image layer not found!")
            .decode()
            .unwrap();
        let joints = image::io::Reader::open(joint_path)
            .expect("joints image layer not found")
            .decode()
            .unwrap();
        let background = nannou::image::io::Reader::open(backround_path)
            .expect("background image layer not found")
            .decode()
            .unwrap();

        let width = points.width() as usize;
        let height = points.height() as usize;
        assert_eq!(joints.width(), points.width());
        assert_eq!(joints.height(), points.height());

        let mut composite = RgbaImage::new(width as u32, height as u32);
        for i in 0..width as u32 {
            for j in 0..height as u32 {
                if points.get_pixel(i, j)[3] != 0 {
                    composite[(i, j)] = Rgba::from(POINT_COLOR);
                } else if joints.get_pixel(i, j)[3] != 0 {
                    composite[(i, j)] = Rgba::from(JOINT_COLOR);
                }
            }
        }

        Self {
            floor,
            composite,
            background,
            visited: vec![vec![false; height]; width],
            regions: Vec::new(),
            points: Vec::new(),
            width,
            height,
            json_data: String::new(),
            save_directory: save_path,
        }
    }

    pub fn composite_image(&self) -> &RgbaImage {
        &self.composite
    }

    pub fn background_image(&self) -> &DynamicImage {
        &self.background
    }

    pub fn points(&self) -> &Vec<Region> {
        &self.points
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn build_data(&mut self) {
        for i in 0..self.width as i32 {
            for j in 0..self.height as i32 {
                if !self.visited[i as usize][j as usize] && self.has_color_at((i, j)) {
                    let cells = self.flood_fill((i, j));
                    self.regions.push(Region::new(cells));
                }
            }
        }

        self.create_connections();

        self.points = self
            .regions
            .iter()
            .filter(|region| self.color_at(region.cells[0]) == POINT_COLOR)
            .cloned()
            .collect();

        let mut json = array![];

        for point in self.points.iter() {
            let (x_pos, y_pos) = point.position();
            let room = object! {
                id: point.hash(),
                number: JsonValue::Null,
                floor: self.floor,
                x: x_pos,
                y: y_pos,
                room_type: "",
                description: "",
                nodes: point.nodes.clone()
            };

            json.push(room).expect("faulty point data!");
        }

        self.json_data = json.pretty(4);
    }

    pub fn json_replace(&mut self, from: &String, to: &String) {
        self.json_data = self.json_data.replace(from, to);
    }

    pub fn save_to_file(&self) {
        std::fs::File::create(&self.save_directory).expect("couldn't create save file!");
        std::fs::write(&self.save_directory, &self.json_data).expect("invalid save directory!");
    }

    pub fn name_image(&mut self) {}

    pub fn in_bounds(&self, (x, y): Point) -> bool {
        x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32
    }

    pub fn color_at(&self, (x, y): Point) -> [u8; 4] {
        self.composite.get_pixel(x as u32, y as u32).0
    }

    pub fn region_at(&self, (x, y): Point) -> &Region {
        self.regions
            .iter()
            .find(|region| region.cells.iter().any(|cell| *cell == (x, y)))
            .expect("no region at given point!")
    }

    pub fn region_index(&self, find: &Region) -> usize {
        self.regions
            .iter()
            .enumerate()
            .find(|(_, region)| region.cells[0] == find.cells[0])
            .unwrap()
            .0
    }

    pub fn has_color_at(&self, (x, y): Point) -> bool {
        self.composite.get_pixel(x as u32, y as u32).0 != [0, 0, 0, 0]
    }

    pub fn flood_fill(&mut self, (x, y): Point) -> Vec<Point> {
        let mut cells = Vec::new();
        let mut queue = vec![(x, y)];
        let color = self.color_at((x, y));
        self.visited[x as usize][y as usize] = true;

        while queue.len() > 0 {
            let cell = queue.pop().unwrap();
            cells.push(cell);

            for i in cell.0 - 1..=cell.0 + 1 {
                for j in cell.1 - 1..=cell.1 + 1 {
                    if self.in_bounds((x, y))
                        && !self.visited[i as usize][j as usize]
                        && self.color_at((i, j)) == color
                    {
                        self.visited[i as usize][j as usize] = true;
                        queue.push((i, j));
                    }
                }
            }
        }

        cells
    }

    pub fn get_adjacent(&self, region: &Region) -> Vec<Region> {
        let mut adjacent_cells: Vec<Point> = Vec::new();
        let mut adjacent_regions: Vec<Region> = Vec::new();
        let color = self.color_at(region.cells[0]);

        for (x, y) in region.cells.iter() {
            for i in x - 1..=x + 1 {
                for j in y - 1..=y + 1 {
                    if self.in_bounds((i, j))
                        && self.has_color_at((i, j))
                        && self.color_at((i, j)) != color
                        && !adjacent_cells.contains(&(i, j))
                    {
                        adjacent_cells.push((i, j));
                    }
                }
            }
        }

        for xy in adjacent_cells {
            if !adjacent_regions
                .iter()
                .any(|region| region.cells.contains(&xy))
            {
                adjacent_regions.push(self.region_at(xy).clone());
            }
        }

        adjacent_regions
    }

    pub fn create_connections(&mut self) {
        let mut node_pairs = Vec::new();

        for (i, point) in self
            .regions
            .iter()
            .enumerate()
            .filter(|(_, region)| self.color_at(region.cells[0]) == POINT_COLOR)
        {
            let joints = self.get_adjacent(point);
            for joint in joints.iter() {
                let adjacents = self.get_adjacent(joint);
                for adjacent in adjacents
                    .iter()
                    .filter(|adjacent| adjacent.cells[0] != point.cells[0])
                {
                    node_pairs.push((i, adjacent.clone()));
                }
            }
        }

        for (i, node) in node_pairs.into_iter() {
            self.regions[i].nodes.push(node.hash());
        }
    }
}
