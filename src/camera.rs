use cgmath::*;
use log::debug;
use num_traits::{clamp, clamp_max, clamp_min};
// use std::f32::consts::FRAC_PI_2;
// use log::debug;
use wgpu::util::DeviceExt;

use crate::{
    component::{self, EntityMap, ParallaxComponent},
    context,
    uniform::WorldUniform,
    utils,
};

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
    screen_to_clip: [[f32; 4]; 4],
    clip_to_screen: [[f32; 4]; 4],
}

impl OrthoUniform {
    pub fn new() -> Self {
        Self {
            screen_to_clip: Matrix4::identity().into(),
            clip_to_screen: Matrix4::identity().into(),
        }
    }

    pub fn calc(
        center: Vector3<f32>,
        width: f32,
        height: f32,
        zfar: f32,
        znear: f32,
    ) -> Matrix4<f32> {
        let two: f32 = 2.0;
        let left = center.x - width / 2.0;
        let right = center.x + width / 2.0;

        let top = center.y + height / 2.0;
        let bottom = center.y - height / 2.0;

        let c0r0 = two / (right - left);
        let c0r1 = f32::zero();
        let c0r2 = f32::zero();
        let c0r3 = f32::zero();

        let c1r0 = f32::zero();
        let c1r1 = -1. * two / (top - bottom);
        let c1r2 = f32::zero();
        let c1r3 = f32::zero();

        let c2r0 = f32::zero();
        let c2r1 = f32::zero();
        let c2r2 = f32::one() / (zfar - znear);
        let c2r3 = f32::zero();

        let c3r0 = -(right + left) / (right - left);
        let c3r1 = (top + bottom) / (top - bottom);
        let c3r2 = -(znear) / (zfar - znear);
        let c3r3 = f32::one();

        #[cfg_attr(rustfmt, rustfmt_skip)]
            Matrix4::new(
                c0r0, c0r1, c0r2, c0r3,
                c1r0, c1r1, c1r2, c1r3,
                c2r0, c2r1, c2r2, c2r3,
                c3r0, c3r1, c3r2, c3r3,
            )
    }

    pub fn resize(&mut self, center: Vector3<f32>, width: f32, height: f32, zfar: f32, znear: f32) {
        let mat = Self::calc(center, width, height, zfar, znear);

        // for some dame reason mat.is_invertible and mat.invert use different equal functions to check
        // if the det is 0, sigh. another reason to stop using cgmath. increasing number so really
        // small values dont get counted as 0
        assert!((mat * 1000.).is_invertible()); // I want to know if this ever happens... lol
        self.screen_to_clip = mat.into();
        self.clip_to_screen = mat.invert().unwrap().into();
    }

    pub fn get_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ortho buffer"),
            contents: bytemuck::cast_slice(&[self.screen_to_clip, self.clip_to_screen]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }
}

#[derive(Clone)]
pub struct OrthographicCamera {
    width: f32,
    height: f32,
    znear: f32,
    zfar: f32,
    center: Vector3<f32>,

    uniform: OrthoUniform,
}

impl OrthographicCamera {
    pub fn original_center(width: f32, height: f32) -> Vector3<f32> {
        vec3(width / 2.0, height / 2.0, 1.0)
    }

    pub fn new_with_pos(
        width: u32,
        height: u32,
        znear: f32,
        zfar: f32,
        center: Vector3<f32>,
    ) -> Self {
        Self {
            width: width as f32,
            height: height as f32,
            znear,
            zfar,
            center,
            uniform: OrthoUniform::new(),
        }
    }

    pub fn new(width: u32, height: u32, znear: f32, zfar: f32) -> Self {
        Self::new_with_pos(
            width,
            height,
            znear,
            zfar,
            Self::original_center(width as f32, height as f32),
        )
    }

    pub fn position(&self) -> Vector3<f32> {
        return self.center;
    }

    pub fn update_position(&mut self, position: Vector3<f32>) {
        self.center = position;
        self.uniform
            .resize(self.center, self.width, self.height, self.zfar, self.znear);
    }

    pub fn update_position_delta(&mut self, position: Vector3<f32>) {
        self.center += position;
        self.uniform
            .resize(self.center, self.width, self.height, self.zfar, self.znear);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let width = width as f32;
        let height = height as f32;
        let disp = self.center - Self::original_center(self.width, self.height);

        self.center = vec3(width / 2., height / 2., 1.0)
            + disp.mul_element_wise(vec3(width / self.width, height / self.height, 1.));
        self.width = width;
        self.height = height;

        self.uniform
            .resize(self.center, width, height, self.zfar, self.znear);
    }

    pub fn get_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        self.uniform.get_buffer(device)
    }
}

pub struct CameraController {}

impl CameraController {
    pub fn update(
        // context: &context::Context,
        player_position_vec: cgmath::Vector2<f32>,
        camera: &mut OrthographicCamera,
        world_uniform: &WorldUniform,
        parallax_components: &mut EntityMap<component::ParallaxComponent>,
        vertex_array_components: &mut EntityMap<component::VertexArrayComponent>,
        position_components: &mut EntityMap<component::PositionComponent>,
    ) {
        let world_uniform_reg: cgmath::Matrix4<f32> = world_uniform.world_to_screen.into();
        let world_uniform_inv: cgmath::Matrix4<f32> = world_uniform.screen_to_world.into();

        let screen_position = world_uniform_reg
            * cgmath::Vector4::new(player_position_vec.x, player_position_vec.y, 1.0, 1.0);

        let original_position =
            cgmath::Vector2::new(camera.width as f32 / 2.0, camera.height as f32 / 2.0);

        let mut update_parallax = |dir: cgmath::Vector2<f32>, camera: &mut OrthographicCamera| {
            utils::zip3_entities_mut(
                parallax_components,
                vertex_array_components,
                position_components,
            )
            .for_each(
                |(_, parallax_component, vertex_array_component, position_component)| {
                    if let (
                        Some(parallax_component),
                        Some(vertex_array_component),
                        Some(position_component),
                    ) = (
                        parallax_component,
                        vertex_array_component,
                        position_component,
                    ) {
                        let new_position_screen_space = cgmath::vec2(
                            clamp_min(camera.position().x, original_position.x),
                            clamp_max(camera.position().y, original_position.y),
                        );

                        vertex_array_component
                            .tex_coords
                            .iter_mut()
                            .zip(vertex_array_component.whole_tex_coords.iter())
                            .for_each(|(coord, whole_coord)| {
                                *coord = (whole_coord
                                    + (new_position_screen_space - original_position)
                                        * parallax_component.move_speed
                                        / 200000.)
                                    % 1.0;
                                // if screen_position.x > camera.width as f32 {
                                //     debug!("{:?}", coord);
                                // }
                            });

                        let new_position_world_space = world_uniform_inv
                            * cgmath::Vector4::new(
                                new_position_screen_space.x,
                                new_position_screen_space.y,
                                1.,
                                1.,
                            );

                        position_component.position = new_position_world_space.xy();
                    }
                },
            )
        };

        if screen_position.x > camera.width as f32 / 2.0 {
            camera.update_position(Vector3::new(
                screen_position.x,
                camera.position().y,
                camera.position().z,
            ));

            update_parallax(cgmath::Vector2::unit_x(), camera);
        }

        if screen_position.y < camera.height as f32 / 2.0 {
            camera.update_position(Vector3::new(
                camera.position().x,
                screen_position.y,
                camera.position().z,
            ));

            // debug!("{:?} {:?}", screen_position, camera.position());

            update_parallax(cgmath::Vector2::unit_y(), camera);
        }
    }
}
