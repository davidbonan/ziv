use super::orientation::Orientation;

/// Upright pixels in the linear working space, row by row from the top-left.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkingImage {
    width: u32,
    height: u32,
    pixels: Vec<[f32; 3]>,
}

impl WorkingImage {
    pub fn new(width: u32, height: u32, pixels: Vec<[f32; 3]>) -> Self {
        assert_eq!(
            pixels.len(),
            width as usize * height as usize,
            "pixel count must match {width}×{height}"
        );
        Self {
            width,
            height,
            pixels,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[[f32; 3]] {
        &self.pixels
    }

    pub fn pixel(&self, x: u32, y: u32) -> [f32; 3] {
        self.pixels[y as usize * self.width as usize + x as usize]
    }

    /// The same pixels turned upright, `self` being stored with `orientation`.
    pub fn upright(self, orientation: Orientation) -> Self {
        if orientation == Orientation::Upright {
            return self;
        }
        let stored_size = [self.width, self.height];
        let (width, height) = if orientation.swaps_width_and_height() {
            (self.height, self.width)
        } else {
            (self.width, self.height)
        };
        let pixels = (0..height)
            .flat_map(|y| (0..width).map(move |x| [x, y]))
            .map(|position| {
                let [x, y] = orientation.stored_position(position, stored_size);
                self.pixel(x, y)
            })
            .collect();
        Self::new(width, height, pixels)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn numbered(width: u32, height: u32) -> WorkingImage {
        let pixels = (0..width * height).map(|index| [index as f32; 3]).collect();
        WorkingImage::new(width, height, pixels)
    }

    fn numbers(image: &WorkingImage) -> Vec<u32> {
        image.pixels().iter().map(|pixel| pixel[0] as u32).collect()
    }

    #[test]
    fn each_orientation_turns_the_stored_pixels_upright() {
        // Stored: 0 1 2
        //         3 4 5
        let cases = [
            (Orientation::Upright, [3, 2], vec![0, 1, 2, 3, 4, 5]),
            (
                Orientation::FlippedHorizontally,
                [3, 2],
                vec![2, 1, 0, 5, 4, 3],
            ),
            (Orientation::Rotated180, [3, 2], vec![5, 4, 3, 2, 1, 0]),
            (
                Orientation::FlippedVertically,
                [3, 2],
                vec![3, 4, 5, 0, 1, 2],
            ),
            (Orientation::Transposed, [2, 3], vec![0, 3, 1, 4, 2, 5]),
            (
                Orientation::Rotated90Clockwise,
                [2, 3],
                vec![3, 0, 4, 1, 5, 2],
            ),
            (Orientation::Transversed, [2, 3], vec![5, 2, 4, 1, 3, 0]),
            (
                Orientation::Rotated270Clockwise,
                [2, 3],
                vec![2, 5, 1, 4, 0, 3],
            ),
        ];

        for (orientation, size, expected) in cases {
            let upright = numbered(3, 2).upright(orientation);

            assert_eq!([upright.width(), upright.height()], size, "{orientation:?}");
            assert_eq!(numbers(&upright), expected, "{orientation:?}");
        }
    }
}
