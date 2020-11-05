#[derive(Debug, Copy, Clone)]
pub struct WindowDim {
    pub width: u32,
    pub height: u32,
}

impl WindowDim {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }
}
