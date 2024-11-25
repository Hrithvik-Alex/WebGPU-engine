use cgmath::*;
// use std::f32::consts::FRAC_PI_2;
// use log::debug;
use wgpu::util::DeviceExt;

// #[rustfmt::skip]
// pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
//     1.0, 0.0, 0.0, 0.0,
//     0.0, 1.0, 0.0, 0.0,
//     0.0, 0.0, 0.5, 0.5,
//     0.0, 0.0, 0.0, 1.0,
// );

// // const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

// // We need this for Rust to store our data correctly for the shaders
// #[repr(C)]
// // This is so we can store this in a buffer
// #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// pub struct CameraUniform {
//     // We can't use cgmath with bytemuck directly, so we'll have
//     // to convert the Matrix4 into a 4x4 f32 array
//     view: [[f32; 4]; 4],
// }

// impl CameraUniform {
//     pub fn new(camera: &Camera) -> Self {
//         Self { view: camera.calc_matrix().into() }
//     }
// }

// pub struct Camera {
//     position: Vector3<f32>,
// }

// impl Camera {

//     pub fn new<
//     V: Into<Vector3<f32>>,

//     >(
//         position: V

//     ) -> Self {
//         Self {
//             position: position.into(),
//     }
// }

//     pub fn calc_matrix(&self) -> cgmath::Matrix4<f32> {
//         let forward_vector = ( self.position).normalize();
//         let up_vector_temp = cgmath::Vector3::unit_y();
//         let right_vector = up_vector_temp.cross(forward_vector);
//         let up_vector = forward_vector.cross(right_vector);

//         #[rustfmt::skip]
//         let look_at_matrix =
//          cgmath::Matrix4::new(
//             right_vector.x, up_vector.x, forward_vector.x, 0.0,
//             right_vector.y, up_vector.y ,forward_vector.y, 0.0,
//            right_vector.z, up_vector.z, forward_vector.z, 0.0,
//             0.0, 0.0, 0.0, 1.0,
//         )
//         *
//         cgmath::Matrix4::new(
//             1.,0.,0.,0.,
//             0.,1.,0.,0.,
//             0.,0.,1.,0.,
//             -self.position.x, -self.position.y, -self.position.z, 1.
//         )
//         ;

// debug!( "look_at_matrix: {:?}", look_at_matrix);

//         return look_at_matrix;

//     }

//     pub fn get_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
//         let camera_uniform = CameraUniform::new(self);
//         device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("camera buffer"),
//             contents: bytemuck::cast_slice(&[camera_uniform]),
//             usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
//         })
//     }

// }

// // We need this for Rust to store our data correctly for the shaders
// #[repr(C)]
// // This is so we can store this in a buffer
// #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// pub struct ProjectionUniform {
//     // We can't use cgmath with bytemuck directly, so we'll have
//     // to convert the Matrix4 into a 4x4 f32 array
//     proj: [[f32; 4]; 4],
// }

// impl ProjectionUniform {
//     pub fn new() -> Self {
//         Self { proj: Matrix4::identity().into() }
//     }

//     pub fn update(&mut self, projection: &Projection) {
//         self.proj = projection.calc_matrix().into();
//     }

// }

// pub struct Projection {
//     width: u32,
//     height: u32,
//     fovy: Rad<f32>,
//     aspect: f32,
//     znear: f32,
//     zfar: f32,
//     is_orthographic: bool,
// }

// // TODO: understand matrix math
// impl Projection {
//     pub fn new<F: Into<Rad<f32>>>(
//         width: u32,
//         height: u32,
//         fovy: F,
//         znear: f32,
//         zfar: f32,
//         is_orthographic: bool,
//     ) -> Self {
//         Self {
//             width,
//             height,
//             aspect: width as f32 / height as f32,
//             fovy: fovy.into(),
//             znear,
//             zfar,
//             is_orthographic,
//         }
//     }

//     pub fn resize(&mut self, width: u32, height: u32) {
//         self.aspect = width as f32 / height as f32;
//     }

//     pub fn calc_matrix(&self) -> Matrix4<f32> {
//         if self.is_orthographic {
//             let two: f32 = 2.0;

//             let c0r0 = two / (self.width as f32);
//             let c0r1 = f32::zero();
//             let c0r2 = f32::zero();
//             let c0r3 = f32::zero();

//             let c1r0 = f32::zero();
//             let c1r1 = two / (self.height as f32);
//             let c1r2 = f32::zero();
//             let c1r3 = f32::zero();

//             let c2r0 = f32::zero();
//             let c2r1 = f32::zero();
//             let c2r2 = f32::one() / (self.zfar - self.znear);
//             let c2r3 = f32::zero();

//             let c3r0 = -1.0;
//             let c3r1 = -1.0;
//             let c3r2 = -(self.znear) / (self.zfar - self.znear);
//             let c3r3 = f32::one();

//             #[cfg_attr(rustfmt, rustfmt_skip)]
//             Matrix4::new(
//                 c0r0, c0r1, c0r2, c0r3,
//                 c1r0, c1r1, c1r2, c1r3,
//                 c2r0, c2r1, c2r2, c2r3,
//                 c3r0, c3r1, c3r2, c3r3,
//             )

//             // OPENGL_TO_WGPU_MATRIX *
//             // ortho(0.0, self.width as f32, 0.0, self.height as f32, self.znear, self.zfar)
//         } else {
//             OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
//         }
//     }
//     pub fn get_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
//         let mut projection_uniform = ProjectionUniform::new();
//         projection_uniform.update(self);
//          device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("projection buffer"),
//             contents: bytemuck::cast_slice(&[projection_uniform]),
//             usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
//         })

//     }
// }

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct OrthoUniform {
    mat: [[f32; 4]; 4],
}

impl OrthoUniform {
    pub fn new() -> Self {
        Self {
            mat: Matrix4::identity().into(),
        }
    }

    pub fn update(&mut self, orthographic_camera: &OrthographicCamera) {
        self.mat = orthographic_camera.get_matrix().into();
    }
}

#[derive(Clone)]
pub struct OrthographicCamera {
    width: u32,
    height: u32,
    znear: f32,
    zfar: f32,
    center: Vector3<f32>,
}

impl OrthographicCamera {
    pub fn new(width: u32, height: u32, znear: f32, zfar: f32, center: Vector3<f32>) -> Self {
        Self {
            width,
            height,
            znear,
            zfar,
            center,
        }
    }

    pub fn position(&self) -> Vector3<f32> {
        return self.center;
    }

    pub fn update_position(&mut self, position: Vector3<f32>) {
        self.center = position;
    }

    pub fn get_matrix(&self) -> Matrix4<f32> {
        let two: f32 = 2.0;
        let left = self.center.x - self.width as f32 / 2.0;
        let right = self.center.x + self.width as f32 / 2.0;

        let top = self.center.y + self.height as f32 / 2.0;
        let bottom = self.center.y - self.height as f32 / 2.0;

        let c0r0 = two / (right - left);
        let c0r1 = f32::zero();
        let c0r2 = f32::zero();
        let c0r3 = f32::zero();

        let c1r0 = f32::zero();
        let c1r1 = two / (top - bottom);
        let c1r2 = f32::zero();
        let c1r3 = f32::zero();

        let c2r0 = f32::zero();
        let c2r1 = f32::zero();
        let c2r2 = f32::one() / (self.zfar - self.znear);
        let c2r3 = f32::zero();

        let c3r0 = -(right + left) / (right - left);
        let c3r1 = -(top + bottom) / (top - bottom);
        let c3r2 = -(self.znear) / (self.zfar - self.znear);
        let c3r3 = f32::one();

        #[cfg_attr(rustfmt, rustfmt_skip)]
            Matrix4::new(
                c0r0, c0r1, c0r2, c0r3,
                c1r0, c1r1, c1r2, c1r3,
                c2r0, c2r1, c2r2, c2r3,
                c3r0, c3r1, c3r2, c3r3,
            )
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn get_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        let mut ortho_uniform = OrthoUniform::new();
        ortho_uniform.update(self);
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ortho buffer"),
            contents: bytemuck::cast_slice(&[ortho_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }
}
