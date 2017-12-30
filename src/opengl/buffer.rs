use glium::Program;
use glium::Surface;
use glium::VertexBuffer;
use glium::backend::Facade;
use glium::framebuffer::SimpleFrameBuffer;
use glium::index::NoIndices;
use glium::program::ProgramCreationInput;
use glium::texture::Texture2d;
use glium::uniforms::{AsUniformValue, UniformValue, Uniforms};
use std;
use std::borrow::Cow;
use std::cell::RefCell;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::Path;
use std::rc::Rc;

use super::renderer::Vertex;
use config::BufferConfig;

pub struct UniformsStorageVec<'name, 'uniform>(Vec<(Cow<'name, str>, Box<AsUniformValue + 'uniform>)>);

impl<'name, 'uniform> UniformsStorageVec<'name, 'uniform> {
    pub fn new() -> Self {
        UniformsStorageVec(Vec::new())
    }

    pub fn push<S, U>(&mut self, name: S, uniform: U)
    where
        S: Into<Cow<'name, str>>,
        U: AsUniformValue + 'uniform,
    {
        self.0.push((name.into(), Box::new(uniform)))
    }
}

impl<'name, 'uniform> Uniforms for UniformsStorageVec<'name, 'uniform> {
    #[inline]
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut output: F) {
        for &(ref name, ref uniform) in &self.0 {
            output(name, uniform.as_uniform_value());
        }
    }
}

pub struct Buffer {
    texture: Texture2d,
    program: Program,
    attachments: Vec<Rc<Texture2d>>,
    depends: Vec<Rc<RefCell<Buffer>>>,
}

impl Buffer {
    pub fn new(
        facade: &Facade, config: &BufferConfig, attachments: Vec<Rc<Texture2d>>
    ) -> Self {
        let file = match File::open(config.vertex.to_string()) {
            Ok(file) => file,
            Err(error) => {
                error!("Could not open vertex shader file: {}", error);
                std::process::exit(1);
            }
        };
        let mut buf_reader = BufReader::new(file);
        let mut vertex_source = String::new();
        match buf_reader.read_to_string(&mut vertex_source) {
            Ok(_) => info!("Using vertex shader: {}", config.vertex),
            Err(error) => {
                error!("Could not read vertex shader file: {}", error);
                std::process::exit(1);
            }
        };

        let file = match File::open(config.fragment.to_string()) {
            Ok(file) => file,
            Err(error) => {
                error!("Could not open fragment shader file: {}", error);
                std::process::exit(1);
            }
        };
        let mut buf_reader = BufReader::new(file);
        let mut fragment_source = String::new();
        match buf_reader.read_to_string(&mut fragment_source) {
            Ok(_) => info!("Using fragment shader: {}", config.fragment),
            Err(error) => {
                error!("Could not read fragment shader file: {}", error);
                std::process::exit(1);
            }
        };

        let input = ProgramCreationInput::SourceCode {
            vertex_shader: &vertex_source,
            tessellation_control_shader: None,
            tessellation_evaluation_shader: None,
            geometry_shader: None,
            fragment_shader: &fragment_source,
            transform_feedback_varyings: None,
            outputs_srgb: true,
            uses_point_size: false,
        };
        let program = Program::new(facade, input);
        let program = match program {
            Ok(program) => program,
            Err(error) => {
                error!("{}", error);
                std::process::exit(1);
            }
        };

        let texture = Texture2d::empty(facade, config.width, config.height).unwrap();

        Buffer {
            texture: texture,
            program: program,
            attachments: attachments,
            depends: Vec::new(),
        }
    }

    pub fn link_depends(&mut self, depends: &mut Vec<Rc<RefCell<Buffer>>>) {
        self.depends.append(depends);
    }

    pub fn render_to<'buf, S>(
        &self, surface: &mut S, vertex_buffer: &VertexBuffer<Vertex>, index_buffer: &NoIndices, time: f32, pointer: [f32; 4]
    ) where
        S: Surface,
    {
        surface.clear_color(0.0, 0.0, 0.0, 1.0);

        let mut uniforms = UniformsStorageVec::new();

        uniforms.push("resolution", surface.get_dimensions());

        uniforms.push("time", time as f32);

        uniforms.push(
            "pointer",
            [
                pointer[0],
                surface.get_dimensions().0 as f32 - pointer[1],
                pointer[2],
                surface.get_dimensions().1 as f32 - pointer[3],
            ],
        );

        for (i, attachment) in self.attachments.iter().enumerate() {
            uniforms.push(format!("texture{}", i), attachment.sampled());
        }

        for (i, buffer) in self.depends.iter().enumerate() {
            buffer.borrow().render_to_self(vertex_buffer, index_buffer, time, pointer);
            uniforms.push(format!("buffer{}", i), buffer.borrow().texture.sampled());
        }

        surface
            .draw(vertex_buffer, index_buffer, &self.program, &uniforms, &Default::default())
            .unwrap();
    }

    pub fn render_to_self(
        &self, vertex_buffer: &VertexBuffer<Vertex>, index_buffer: &NoIndices, time: f32, pointer: [f32; 4]
    ) {
        self.render_to(&mut self.texture.as_surface(), vertex_buffer, index_buffer, time, pointer);
    }

    pub fn resize(&mut self, facade: &Facade, width: u32, height: u32) {
        self.texture = Texture2d::empty(facade, width, height).unwrap();
    }
}
