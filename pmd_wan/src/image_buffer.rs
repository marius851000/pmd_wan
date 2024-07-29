/// Represents an paletted image, with each pixel being an 8bits number.
///
/// Images with no pixel are valid, and it is guarantted that width*height == buffer.len()
#[derive(Debug, PartialEq, Eq)]
pub struct ImageBuffer {
    buffer: Vec<u8>,
    width: u16,
    height: u16,
}

impl ImageBuffer {
    pub fn new_from_vec(buffer: Vec<u8>, width: u16, height: u16) -> Option<ImageBuffer> {
        if width as usize * height as usize != buffer.len() {
            return None;
        }
        Some(Self {
            buffer,
            width,
            height,
        })
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn buffer(&self) -> &Vec<u8> {
        &self.buffer
    }

    pub fn have_pixel(&self) -> bool {
        self.width != 0 || self.height != 0
    }

    pub fn get_pixel(&self, x: u16, y: u16) -> Option<u8> {
        if x >= self.width {
            return None;
        }
        self.buffer
            .get(y as usize * self.width as usize + x as usize)
            .copied()
    }

    pub fn cut_top(&mut self) -> usize {
        if !self.have_pixel() {
            return 0;
        }
        let mut number_of_line_to_cut: u16 = 0;
        for row in self.buffer.chunks_exact(self.width as usize) {
            let mut have_element = false;
            for pixel in row {
                have_element |= *pixel != 0;
            }
            if have_element {
                break;
            } else {
                number_of_line_to_cut += 1;
            }
        }
        let buffer = &self.buffer
            [number_of_line_to_cut as usize * self.width as usize..self.buffer.len()]
            .to_vec();
        self.buffer = buffer.clone();
        self.height -= number_of_line_to_cut;
        number_of_line_to_cut.into()
    }

    pub fn cut_bottom(&mut self) -> usize {
        if !self.have_pixel() {
            return 0;
        }
        let mut number_of_cut_line = 0;
        'main: for line_nb in (0..self.height).rev() {
            for pixel_nb in (line_nb * self.width)..((line_nb + 1) * self.width) {
                //no panic: pixel_nb should always be in the appropriate range
                if self.buffer[pixel_nb as usize] != 0 {
                    break 'main;
                };
            }
            number_of_cut_line += 1;
            self.height -= 1;
            self.buffer
                .truncate(self.height as usize * self.width as usize);
        }
        number_of_cut_line
    }

    pub fn cut_right(&mut self) -> usize {
        if !self.have_pixel() {
            return 0;
        }
        let mut number_of_cut_row = 0;
        'main: for _row in (0..self.width).rev() {
            for line in 0..(self.height as usize) {
                let pixel_id = self.width as usize * line + self.width as usize - 1;
                if self.buffer[pixel_id] != 0 {
                    break 'main;
                };
            }
            for line in (0..(self.height as usize)).rev() {
                let pixel_to_remove = self.width as usize * line + self.width as usize - 1;
                self.buffer.remove(pixel_to_remove);
            }
            self.width -= 1;
            number_of_cut_row += 1;
        }
        number_of_cut_row
    }

    pub fn cut_left(&mut self) -> usize {
        if !self.have_pixel() {
            return 0;
        }
        let mut number_of_cut_row = 0;
        'main: for _row in (0..self.width).rev() {
            for line in 0..(self.height as usize) {
                let pixel_id = self.width as usize * line;
                if self.buffer[pixel_id] != 0 {
                    break 'main;
                };
            }
            for line in (0..self.height).rev() {
                let pixel_to_remove = self.width as usize * line as usize;
                self.buffer.remove(pixel_to_remove);
            }
            self.width -= 1;
            number_of_cut_row += 1;
        }
        number_of_cut_row
    }

    pub fn get_fragment(
        &self,
        start_x: u16,
        start_y: u16,
        width: u16,
        height: u16,
        default: u8,
    ) -> ImageBuffer {
        let mut buffer = Vec::new();
        for y in start_y..start_y + height {
            for x in start_x..start_x + width {
                buffer.push(self.get_pixel(x, y).unwrap_or(default));
            }
        }
        ImageBuffer::new_from_vec(buffer, width, height).unwrap()
    }
}
