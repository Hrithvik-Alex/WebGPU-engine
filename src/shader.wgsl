struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal_coords: vec2<f32>,
    @location(3) texture: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal_coords: vec2<f32>,
    @location(2) texture: u32,

};

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

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.normal_coords = model.normal_coords;
    out.clip_position =  camera.view_proj * /*camera.view *  */  (world.matrix * vec4<f32>(model.position, 1.0));
    out.texture = model.texture;
    return out;
}

struct LightUniform {
    position: vec3<f32>,
    intensity: f32,
    color: vec3<f32>,
};

@group(1) @binding(0)
var<storage> light_uniforms: array<LightUniform>;
@group(1) @binding(1)
var<uniform> light_len: u32;

@group(2) @binding(0)
var t_character: texture_2d<f32>;
@group(2) @binding(1)
var n_character: texture_2d<f32>;
@group(2) @binding(2)
var s_character: sampler;

@group(3) @binding(0)
var t_minotaur: texture_2d<f32>;
@group(3) @binding(1)
var n_minotaur: texture_2d<f32>;
@group(3) @binding(2)
var s_minotaur: sampler;

@group(4) @binding(0)
var t_bg: texture_2d<f32>;
@group(4) @binding(1)
var s_bg: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var ambient_light_intensity = 0.2;


    // var light1_pos = vec4<f32>(0.5,0.5,1.0,0.0);
    // var light1_color = vec4<f32>(10000.0,0.0,0.0,0.0);
    // var light1_dist = distance(light1_pos , in.clip_position);
    // var light1_dir = normalize(light1_pos - in.clip_position);


    var color: vec4<f32>;
    var normal: vec4<f32> = vec4(0.);
    switch in.texture {
        case 0u: {
            color = textureSample(t_character, s_character, in.tex_coords);
            normal = textureSample(n_character, s_character, in.tex_coords); 
        }

        case 1u: {
            color = textureSample(t_minotaur, s_character, in.tex_coords);
            normal = textureSample(n_minotaur, s_character, in.tex_coords);
        }

        case 2u: {
            color = textureSample(t_bg, s_bg, in.tex_coords);
        }
        default: {
            color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
        }
    }

    var light = ambient_light_intensity;

    for(var i = 0; i < light_len; i++) {
        var light = light_uniforms[i];
        var light_dir = vec4(light.position, 1.0) - clip_position;
        var light_mag = dot(normal, light_dir);
        var diff_light = light.intensity * max(light_mag, 0.);
    }



    // var light1_final = dot(normal, light1_dir) * light1_color / light1_dist;
    return vec4<f32>(color.xyz  * light, color.w);
}
 
 