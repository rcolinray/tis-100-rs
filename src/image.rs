//! Constructs for generating images using the TIS-100.

/// The colors that the TIS-100 can generate.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Color {
    Black,
    DarkGrey,
    BrightGrey,
    White,
    Red,
}

impl Color {
    /// Get the color for the given integer representation.
    fn from_isize(value: isize) -> Color {
        match value {
            1 => DarkGrey,
            2 => BrightGrey,
            3 => White,
            4 => Red,
            _ => Black,
        }
    }
}

use self::Color::*;

/// The operational modes of the image.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum ImageMode {
    Move,
    Paint,
}

use self::ImageMode::*;

/// An image that can receive values from the TIS-100. When in the `Move` mode, the image receives
/// coordinates that tell it where to draw. When in the `Paint` mode, the image will draw values.
/// Sending a negative value at any time will reset the image to `Move` mode.
#[derive(Debug)]
pub struct Image {
    width: usize,
    height: usize,
    data: Vec<Color>,
    mode: ImageMode,
    position: Vec<isize>,
    offset: usize,
}

impl Image {
    /// Construct a new, empty `Image` with the given width and height.
    pub fn new(width: usize, height: usize) -> Image {
        let mut data = Vec::with_capacity(width * height);
        for _ in 0..width * height {
            data.push(Black);
        }

        Image {
            width: width,
            height: height,
            data: data,
            mode: Move,
            position: Vec::new(),
            offset: 0,
        }
    }

    /// Construct a new `Image` with the given width, height, and initial values.
    pub fn with_data(data: &Vec<isize>, width: usize, height: usize) -> Image {
        assert_eq!(data.len(), width * height);

        let data = data.iter().map(|&i| Color::from_isize(i)).collect::<Vec<_>>();

        Image {
            width: width,
            height: height,
            data: data,
            mode: Move,
            position: Vec::new(),
            offset: 0,
        }
    }

    /// Retrieve the image's data.
    pub fn data(&self) -> &Vec<Color> {
        &self.data
    }

    /// Write a value to the image. If the image is in `Move` mode, then the value will be
    /// interpreted as a coordinate. If the image is in `Paint` mode, then the value will be
    /// interpreted as a color unless the value is negative.
    pub fn write(&mut self, value: isize) {
        if value < 0 {
            self.position.clear();
            self.mode = Move;
            self.offset = 0;
            return;
        }

        if self.mode == Move {
            self.position.push(value);

            if self.position.len() == 2 {
                self.mode = Paint;
            }

            return;
        }

        let row_off = self.position[0] as usize * self.width;

        if row_off < self.width * self.height {
            let col = self.position[1] as usize + self.offset;

            if col < self.width {
                self.data[row_off + col] = Color::from_isize(value);
            }
        }

        self.offset += 1;
    }
}

impl PartialEq for Image {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Eq for Image {}

#[test]
fn test_color_from_isize() {
    assert_eq!(Color::from_isize(0), Black);
    assert_eq!(Color::from_isize(1), DarkGrey);
    assert_eq!(Color::from_isize(2), BrightGrey);
    assert_eq!(Color::from_isize(3), White);
    assert_eq!(Color::from_isize(4), Red);
    assert_eq!(Color::from_isize(5), Black);
}

#[test]
fn test_image_with_data() {
    let expected = vec![DarkGrey, BrightGrey, White, Red];
    let data = vec![1, 2, 3, 4];
    let image = Image::with_data(&data, 2, 2);

    assert_eq!(expected, image.data().clone());
}

#[test]
fn test_image_write() {
    let expected = vec![DarkGrey, BrightGrey, White, Red];
    let mut image = Image::new(2, 2);

    image.write(0);
    image.write(0);
    image.write(1);
    image.write(2);
    image.write(-1);
    image.write(1);
    image.write(0);
    image.write(3);
    image.write(4);
    image.write(-1);

    assert_eq!(expected, image.data().clone());
}
