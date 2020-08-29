// mod fragment;
// mod geometry;
// mod vertex;

// pub use fragment::FragmentShader;
// pub use geometry::GeometryShader;
// pub use vertex::VertexShader;

#[allow(unused)]
macro_rules! shader {
	($name: ident [$gl: ident]: $($ext:tt)+) => {
		pub struct $name {
			id: u32,
			source: String,
		}

		impl $name {
			pub(super) fn handle(&self) -> u32 {
				self.id
			}

			pub unsafe fn from_file(path: &std::path::Path) -> Result<Self, String> {
				use std::{io::Read, ptr, fs::File};
				if let Ok(mut file) = File::open(path) {
					let mut source = String::new();
					// Open the file
					if file.read_to_string(&mut source).is_ok() {
						// Create a new geometry shader
						let id = gl::CreateShader(gl::$gl);
						// Attach the source code to it
						gl::ShaderSource(
							id,
							1,
							&string_to_cstring(source.as_str()).as_ptr(),
							ptr::null(),
						);
						gl::CompileShader(id); // Compile it

						// Checking shader compile status
						let mut status = 0;
						gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut status);
						if status == 0 {
							// Get the legth of the info log
							let mut len = 0;
							gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
							// Allocate the memory to store the log
							let log = new_cstring_with_len(len as usize);
							// Retrive the info log
							gl::GetShaderInfoLog(id, len, &mut len, log.as_ptr() as *mut _);
							if let Ok(string) = log.into_string() {
								Err(format!(concat!(stringify!($($ext),+), " shader compile error:\n{}"), string))
							} else {
								Err(concat!(stringify!($($ext),+),
								" shader compile error:\n<Can't convert the error log to a String>").to_string())
							}
						} else {
							Ok(Self { id, source })
						}
					} else {
						Err("File read failed!".to_string())
					}
				} else {
					Err("Shader not found!".to_string())
				}
			}
		}

		impl std::fmt::Debug for $name {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				write!(f, concat!(stringify!($($ext),+)," source code:\n{}"), self.source)
			}
		}

		impl Drop for $name {
			fn drop(&mut self) {
				unsafe {
					gl::DeleteShader(self.id);
				}
			}
		}
	};
}

shader! {VertexShader[VERTEX_SHADER]: Vertex}
shader! {FragmentShader[FRAGMENT_SHADER]: Fragment}
shader! {GeometryShader[GEOMETRY_SHADER]: Geometry}

pub use program::Program;
mod program;

use std::ffi::CString;

// Creates a CString with the specified length
pub fn new_cstring_with_len(len: usize) -> CString {
    let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
    buffer.extend([b' '].iter().cycle().take(len as usize));
    unsafe { CString::from_vec_unchecked(buffer) }
}

// Converts a String into a CString
pub fn string_to_cstring(string: &str) -> CString {
    unsafe { CString::from_vec_unchecked(string.as_bytes().to_vec()) }
}
