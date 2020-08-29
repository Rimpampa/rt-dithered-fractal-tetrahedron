use super::DepthBuffer;
use super::Texture;

pub struct Framebuffer {
    id: u32,
}

impl Framebuffer {
    pub unsafe fn new(tex: &Texture, depth: Option<&DepthBuffer>) -> Result<Self, String> {
        let mut id = 0;
        gl::GenFramebuffers(1, &mut id);
        gl::BindFramebuffer(gl::FRAMEBUFFER, id);
        gl::FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            tex.handle(),
            0,
        );

        if let Some(buffer) = depth {
            gl::FramebufferRenderbuffer(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::RENDERBUFFER,
                buffer.handle(),
            );
        }

        let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
        if status != gl::FRAMEBUFFER_COMPLETE {
            return Err(format!("Framebuffer incomplete error: {}", status));
        }
        Self::unbind();
        Ok(Framebuffer { id })
    }

    pub unsafe fn bind(framebuffer: &Self) {
        gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer.id);
    }

    pub unsafe fn unbind() {
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.id);
        }
    }
}
