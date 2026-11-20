
use std::ffi::{c_void, CString};
use std::ptr;

#[link(name = "EGL")]
unsafe extern "C" {
    fn eglGetProcAddress(name: *const i8) -> *const c_void;
}

pub struct Renderer {
    pub frame_count: u64,
    pub delta_time: f32,
    program: u32,
    vao: u32,
    initialized: bool,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            frame_count: 0,
            delta_time: 0.0,
            program: 0,
            vao: 0,
            initialized: false,
        }
    }

    pub fn initialize_gl(&mut self) -> bool {
        unsafe {
            gl::load_with(|name| {
                let name = match CString::new(name) {
                    Ok(value) => value,
                    Err(_) => return ptr::null(),
                };

                eglGetProcAddress(name.as_ptr())
            });

            if !gl::GetString::is_loaded()
                || !gl::CreateShader::is_loaded()
                || !gl::ShaderSource::is_loaded()
                || !gl::CompileShader::is_loaded()
                || !gl::CreateProgram::is_loaded()
                || !gl::LinkProgram::is_loaded()
                || !gl::GenVertexArrays::is_loaded()
                || !gl::DrawArrays::is_loaded()
            {
                return false;
            }

            let version = gl::GetString(gl::VERSION);

            if version.is_null() {
                return false;
            }

            let vertex_source = r#"
                #version 300 es
                precision mediump float;

                const vec2 positions[3] = vec2[3](
                    vec2(0.0, 0.65),
                    vec2(-0.65, -0.55),
                    vec2(0.65, -0.55)
                );

                void main() {
                    gl_Position = vec4(
                        positions[gl_VertexID],
                        0.0,
                        1.0
                    );
                }
            "#;

            let fragment_source = r#"
                #version 300 es
                precision mediump float;

                out vec4 fragmentColor;

                void main() {
                    fragmentColor = vec4(
                        0.95,
                        0.95,
                        0.95,
                        1.0
                    );
                }
            "#;

            let vertex = match Self::compile_shader(
                gl::VERTEX_SHADER,
                vertex_source,
            ) {
                Some(shader) => shader,
                None => return false,
            };

            let fragment = match Self::compile_shader(
                gl::FRAGMENT_SHADER,
                fragment_source,
            ) {
                Some(shader) => shader,
                None => {
                    gl::DeleteShader(vertex);
                    return false;
                }
            };

            let program = gl::CreateProgram();

            gl::AttachShader(program, vertex);
            gl::AttachShader(program, fragment);
            gl::LinkProgram(program);

            gl::DeleteShader(vertex);
            gl::DeleteShader(fragment);

            let mut success = 0;

            gl::GetProgramiv(
                program,
                gl::LINK_STATUS,
                &mut success,
            );

            if success == 0 {
                gl::DeleteProgram(program);
                return false;
            }

            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);

            if vao == 0 {
                gl::DeleteProgram(program);
                return false;
            }

            self.program = program;
            self.vao = vao;
            self.initialized = true;

            gl::ClearColor(0.08, 0.08, 0.08, 1.0);

            true
        }
    }

    unsafe fn compile_shader(
        shader_type: u32,
        source: &str,
    ) -> Option<u32> {
        let shader = gl::CreateShader(shader_type);

        if shader == 0 {
            return None;
        }

        let source = match CString::new(source) {
            Ok(value) => value,
            Err(_) => {
                gl::DeleteShader(shader);
                return None;
            }
        };

        let source_ptr = source.as_ptr();

        gl::ShaderSource(
            shader,
            1,
            &source_ptr,
            ptr::null(),
        );

        gl::CompileShader(shader);

        let mut success = 0;

        gl::GetShaderiv(
            shader,
            gl::COMPILE_STATUS,
            &mut success,
        );

        if success == 0 {
            gl::DeleteShader(shader);
            return None;
        }

        Some(shader)
    }

    pub fn resize(&mut self, width: i32, height: i32) -> bool {
        if !self.initialized || width <= 0 || height <= 0 {
            return false;
        }

        unsafe {
            gl::Viewport(0, 0, width, height);
        }

        true
    }

    pub fn render(&mut self) -> bool {
        if !self.initialized {
            return false;
        }

        unsafe {
            gl::Clear(
                gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT,
            );

            gl::UseProgram(self.program);
            gl::BindVertexArray(self.vao);

            gl::DrawArrays(
                gl::TRIANGLES,
                0,
                3,
            );

            gl::BindVertexArray(0);
            gl::UseProgram(0);
        }

        self.frame_count += 1;
        true
    }

    pub fn update(&mut self, delta_time: f32) {
        self.delta_time = delta_time.max(0.0);
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    pub fn shutdown(&mut self) {
        if !self.initialized {
            return;
        }

        unsafe {
            gl::DeleteProgram(self.program);
            gl::DeleteVertexArrays(1, &self.vao);
        }

        self.program = 0;
        self.vao = 0;
        self.initialized = false;
    }
}
