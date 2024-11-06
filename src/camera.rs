use cgmath::*;
// use std::f32::consts::FRAC_PI_2;
use wgpu::util::DeviceExt;
use log::debug;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

// const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;


// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new(camera: &Camera) -> Self {
        Self { view: camera.calc_matrix().into() }
    }
}

pub struct Camera {
    position: Vector3<f32>,
}

impl Camera {

    pub fn new<
    V: Into<Vector3<f32>>,

    >(
        position: V

    ) -> Self {
        Self {
            position: position.into(),
    }
}



    pub fn calc_matrix(&self) -> cgmath::Matrix4<f32> {
        let forward_vector = ( self.position).normalize();
        let up_vector_temp = cgmath::Vector3::unit_y();
        let right_vector = up_vector_temp.cross(forward_vector);
        let up_vector = forward_vector.cross(right_vector);

        #[rustfmt::skip]
        let look_at_matrix =
         cgmath::Matrix4::new(
            right_vector.x, up_vector.x, forward_vector.x, 0.0,
            right_vector.y, up_vector.y ,forward_vector.y, 0.0,
           right_vector.z, up_vector.z, forward_vector.z, 0.0,
            0.0, 0.0, 0.0, 1.0,
        )
        * 
        cgmath::Matrix4::new(
            1.,0.,0.,0.,
            0.,1.,0.,0.,
            0.,0.,1.,0.,
            -self.position.x, -self.position.y, -self.position.z, 1.
        ) 
        ;

debug!( "look_at_matrix: {:?}", look_at_matrix);


        return look_at_matrix;
       
    }

    pub fn get_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        let camera_uniform = CameraUniform::new(self);
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

}



// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ProjectionUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    proj: [[f32; 4]; 4],
}

impl ProjectionUniform {
    pub fn new() -> Self {
        Self { proj: Matrix4::identity().into() }
    }

    pub fn update(&mut self, projection: &Projection) {
        self.proj = projection.calc_matrix().into();
    }


}

pub struct Projection {
    width: u32,
    height: u32,
    fovy: Rad<f32>,
    aspect: f32,
    znear: f32,
    zfar: f32,
    is_orthographic: bool,
}

// TODO: understand matrix math
impl Projection {
    pub fn new<F: Into<Rad<f32>>>(
        width: u32,
        height: u32,
        fovy: F,
        znear: f32,
        zfar: f32,
        is_orthographic: bool,
    ) -> Self {
        Self {  
            width,
            height,
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
            is_orthographic,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        if self.is_orthographic {
            OPENGL_TO_WGPU_MATRIX * ortho(0.0, self.width as f32, 0.0, self.height as f32, self.znear, self.zfar)
        } else {
            OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
        }
    }
    pub fn get_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        let mut projection_uniform = ProjectionUniform::new();
        projection_uniform.update(self);
         device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("projection buffer"),
            contents: bytemuck::cast_slice(&[projection_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })

    }
}


