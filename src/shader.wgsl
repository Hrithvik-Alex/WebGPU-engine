struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal_coords: vec2<f32>,
    @location(3) extra_info: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal_coords: vec2<f32>,
    @location(2) extra_info: u32,
    @location(3) world_position: vec4<f32>,
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
    out.world_position =(world.matrix * vec4<f32>(model.position, 1.0)); 
    out.extra_info = model.extra_info;
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
var pixel_sampler: sampler;

@group(2) @binding(1)
var t_character: texture_2d<f32>;
@group(2) @binding(2)
var n_character: texture_2d<f32>;

@group(2) @binding(3)
var t_minotaur: texture_2d<f32>;
@group(2) @binding(4)
var n_minotaur: texture_2d<f32>;

@group(2) @binding(5)
var t_bg: texture_2d<f32>;


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var ambient_light_intensity = 0.0;


    // var light1_pos = vec4<f32>(0.5,0.5,1.0,0.0);
    // var light1_color = vec4<f32>(10000.0,0.0,0.0,0.0);
    // var light1_dist = distance(light1_pos , in.clip_position);
    // var light1_dir = normalize(light1_pos - in.clip_position);


    var color: vec4<f32>;
    var normal: vec4<f32> = vec4(0.);

    var texture_num = in.extra_info & 255;
    var is_flipped = (in.extra_info & (1u << 8)) != 0;
    var has_normal = false;
    switch texture_num {
        case 0u: {
            color = textureSample(t_character, pixel_sampler, in.tex_coords);
            normal = textureSample(n_character, pixel_sampler, in.tex_coords); 
            has_normal = true;
        }

        case 1u: {
            color = textureSample(t_minotaur, pixel_sampler, in.tex_coords);
            normal = textureSample(n_minotaur, pixel_sampler, in.tex_coords);
            has_normal = true;
        }

        case 2u: {
            color = textureSample(t_bg, pixel_sampler, in.tex_coords);
            // normal = vec4(0.,0,1,0);
            // has_normal = true;
 
        }
        default: {
            color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
        }
    }

    if (is_flipped && has_normal) {
        normal = vec4((1. - normal.x) * normal.w, normal.y, normal.z, normal.w);

                // normal = vec4(normal.x, 0., 0., normal.w);
    }

    var total_light = vec3(ambient_light_intensity);

    for(var i: u32 = 0; i < light_len; i++) {
        var light = light_uniforms[i];

        var light_pos =   (world.matrix * vec4<f32>(light.position, 1.0)); 
        var light_dir = light_pos - in.world_position;
        var light_mag = dot(select(vec4(.25), normalize(normal), has_normal), normalize(light_dir));
        var light_str = light.intensity * max(light_mag, 0.);
        var dist = length(light_dir);
        total_light += ( light_str * light.color / dist ) +  ( 0.2 * light.color / dist );;
        // total_light += (light.intensity *  light.color / dist );
        // if(dist < 100) {
        //     total_light += 1.0;
        // }
    }



    // var light1_final = dot(normal, light1_dir) * light1_color / light1_dist;
    return vec4<f32>(color.xyz + total_light, color.w);
    // return vec4<f32>(total_light, color.w);
    // return normal;
}
 
 