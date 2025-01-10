struct CameraUniform {
    view_proj: mat4x4<f32>,
};

struct WorldUniform {
    matrix: mat4x4<f32>,
};

// struct ProjectionUniform {
//     proj: mat4x4<f32>,
// };

@group(0) @binding(0) 
var<uniform> camera: CameraUniform;

@group(0) @binding(1) 
var<uniform> world: WorldUniform;
// @group(1) @binding(1) // 2.
// var<uniform> projection: ProjectionUniform;
@group(0) @binding(2)
var<uniform> screen_resolution: vec2<f32>;