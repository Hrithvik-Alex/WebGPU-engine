@group(1) @binding(0)
var pixel_sampler: sampler;

@group(1) @binding(1)
var t_character: texture_2d<f32>;
// @group(1) @binding(2)
// var n_character: texture_2d<f32>;

@group(1) @binding(2)
var t_minotaur: texture_2d<f32>;
// @group(2) @binding(4)
// var n_minotaur: texture_2d<f32>;

@group(1) @binding(3)
var t_bg1: texture_2d<f32>;

@group(1) @binding(4)
var t_bg2: texture_2d<f32>;

@group(1) @binding(5)
var t_bg3: texture_2d<f32>;

@group(1) @binding(6)
var t_bg4: texture_2d<f32>;

@group(1) @binding(7)
var t_signpost: texture_2d<f32>;

@group(1) @binding(8)
var t_terrain: texture_2d<f32>;

struct TextureInfo {
    color: vec4<f32>,
    normal: vec4<f32>,
}

fn get_texture_color(extra_info: u32, tex_coords: vec2<f32>) -> TextureInfo {
    var color: vec4<f32>;
    var normal: vec4<f32> = vec4(0.);

    var texture_num = extra_info & 255;
    var has_normal = false;
    switch texture_num {
        case 0u: {
            color = textureSample(t_character, pixel_sampler, tex_coords);
            // normal = textureSample(n_character, pixel_sampler, tex_coords); 
            // has_normal = true;
        }

        case 1u: {
            color = textureSample(t_minotaur, pixel_sampler, tex_coords);
            // normal = textureSample(n_minotaur, pixel_sampler, tex_coords);
            // has_normal = true;
        }

        case 2u: {
            color = textureSample(t_bg1, pixel_sampler, tex_coords);
            // color = vec4(0.1,0.1,0.1,1);
            // normal = vec4(0.,0,1,0);
            // has_normal = true;
 
        }

        case 3u: {
            color = textureSample(t_bg2, pixel_sampler, tex_coords);
            // color = vec4(0.1,0.1,0.1,1);
            // normal = vec4(0.,0,1,0);
            // has_normal = true;
 
        }

        case 4u: {
            color = textureSample(t_bg3, pixel_sampler, tex_coords);
            // color = vec4(0.1,0.1,0.1,1);
            // normal = vec4(0.,0,1,0);
            // has_normal = true;
 
        }

        case 5u: {
            color = textureSample(t_bg4, pixel_sampler, tex_coords);
            // color = vec4(0.1,0.1,0.1,1);
            // normal = vec4(0.,0,1,0);
            // has_normal = true;
 
        }

        case 6u: {
            color = textureSample(t_signpost, pixel_sampler, tex_coords);
            // color = vec4(0.1,0.1,0.1,1);
            // normal = vec4(0.,0,1,0);
            // has_normal = true;
 
        }

        case 7u: {
            color = textureSample(t_terrain, pixel_sampler, tex_coords);
            // color = vec4(0.1,0.1,0.1,1);
            // normal = vec4(0.,0,1,0);
            // has_normal = true;
 
        }
        default: {
            color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
        }
    }


    var is_flipped = (extra_info & (1u << 8)) != 0;

    if (is_flipped ) {
        normal = vec4((1. - normal.x) * normal.w, normal.y, normal.z, normal.w);

                // normal = vec4(normal.x, 0., 0., normal.w);
    }

    var t_info: TextureInfo;
    t_info.color = color;
    t_info.normal = normal;
    return t_info;
}