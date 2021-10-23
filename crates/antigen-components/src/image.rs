#[derive(
    Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub struct Image {
    #[serde(skip)]
    data: Vec<u8>,
    width: u32,
    height: u32,
}

impl Image {
    pub fn new(data: Vec<u8>, width: u32, height: u32) -> Self {
        Image {
            data,
            width,
            height,
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn inverse(&self) -> Image {
        let data = self.data.iter().copied().map(|v| 255 - v).collect::<Vec<_>>();

        Image {
            data,
            width: self.width,
            height: self.height,
        }
    }

    pub fn mandelbrot_r8(size: u32) -> Self {
        let data = (0..size * size)
            .map(|id| {
                let cx = 3.0 * (id % size) as f32 / (size - 1) as f32 - 2.0;
                let cy = 2.0 * (id / size) as f32 / (size - 1) as f32 - 1.0;
                let (mut x, mut y, mut count) = (cx, cy, 0);
                while count < 0xFF && x * x + y * y < 4.0 {
                    let old_x = x;
                    x = x * x - y * y + cx;
                    y = 2.0 * old_x * y + cy;
                    count += 1;
                }
                count
            })
            .collect();

        Image {
            data,
            width: size,
            height: size,
        }
    }

    pub fn julia_set_rgba8(size: u32, cx: f32, cy: f32) -> Self {
        let data = (0..size * size)
            .flat_map(|id| {
                let mut x = 4.0 * (id % size) as f32 / (size - 1) as f32 - 2.0;
                let mut y = 2.0 * (id / size) as f32 / (size - 1) as f32 - 1.0;
                let mut count = 0;
                while count < 0xFF && x * x + y * y < 4.0 {
                    let old_x = x;
                    x = x * x - y * y + cx;
                    y = 2.0 * old_x * y + cy;
                    count += 1;
                }
                std::iter::once(0xFF - (count * 2) as u8)
                    .chain(std::iter::once(0xFF - (count * 5) as u8))
                    .chain(std::iter::once(0xFF - (count * 13) as u8))
                    .chain(std::iter::once(std::u8::MAX))
            })
            .collect();

        Image {
            data,
            width: size,
            height: size,
        }
    }
}
