use gl::types as gl_t;

/// A renderbuffer with only the depth component
pub struct DepthBuffer {
    id: u32,
    width: u32,
    height: u32,
}

impl DepthBuffer {
    pub unsafe fn new(width: u32, height: u32) -> Self {
        let mut id: gl_t::GLuint = 0;
        // Genereate a new renderbuffer
        gl::GenRenderbuffers(1, &mut id);
        gl::BindRenderbuffer(gl::RENDERBUFFER, id);
        gl::RenderbufferStorage(
            gl::RENDERBUFFER,
            gl::DEPTH_COMPONENT,
            width as i32,
            height as i32,
        );
        gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
        Self { id, width, height }
    }

    pub unsafe fn resize(&mut self, width: u32, height: u32) {
        gl::BindRenderbuffer(gl::RENDERBUFFER, self.id);
        gl::RenderbufferStorage(
            gl::RENDERBUFFER,
            gl::DEPTH_COMPONENT,
            width as i32,
            height as i32,
        );
        gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
        self.width = width;
        self.height = height;
    }

    pub(super) unsafe fn handle(&self) -> u32 {
        self.id
    }
}
