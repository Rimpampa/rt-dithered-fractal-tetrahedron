#![allow(dead_code)]
///! This is a thin wrapper around basic OpenGL calls that doesn't guarantee
///! any safety (thus everything is unsafe)
mod depth_buffer;
mod framebuffer;
mod shader;
mod texture;
mod vao;
mod vbo;

pub use depth_buffer::DepthBuffer;
pub use framebuffer::Framebuffer;
pub use shader::{FragmentShader, GeometryShader, Program, VertexShader};
pub use texture::Texture;
pub use vao::VertexArrayObject;
pub use vbo::VertexBufferObject;
