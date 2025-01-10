//#include uniform.wgsl
//#include texture.wgsl

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
    linear_dropoff: f32,
    color: vec3<f32>,
    quadratic_dropoff: f32,
    ambient_strength: f32,
    diffuse_strength: f32,
    _padding: vec2<f32>
};

@group(2) @binding(0)
var<storage> light_uniforms: array<LightUniform>;
@group(2) @binding(1)
var<uniform> light_len: u32;




@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {


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
            // normal = textureSample(n_minotaur, pixel_sampler, in.tex_coords);
            // has_normal = true;
        }

        case 2u: {
            color = textureSample(t_bg1, pixel_sampler, in.tex_coords);
            // color = vec4(0.1,0.1,0.1,1);
            // normal = vec4(0.,0,1,0);
            // has_normal = true;
 
        }

        case 3u: {
            color = textureSample(t_bg2, pixel_sampler, in.tex_coords);
            // color = vec4(0.1,0.1,0.1,1);
            // normal = vec4(0.,0,1,0);
            // has_normal = true;
 
        }

        case 4u: {
            color = textureSample(t_bg3, pixel_sampler, in.tex_coords);
            // color = vec4(0.1,0.1,0.1,1);
            // normal = vec4(0.,0,1,0);
            // has_normal = true;
 
        }

        case 5u: {
            color = textureSample(t_bg4, pixel_sampler, in.tex_coords);
            // color = vec4(0.1,0.1,0.1,1);
            // normal = vec4(0.,0,1,0);
            // has_normal = true;
 
        }
        default: {
            color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
        }
    }
   if (color.w == 0 ) {
    discard;
   }


    if (is_flipped && has_normal) {
        normal = vec4((1. - normal.x) * normal.w, normal.y, normal.z, normal.w);

                // normal = vec4(normal.x, 0., 0., normal.w);
    }

    var ambient_light_intensity = 0.2;
    var total_light = vec3(ambient_light_intensity);


    for(var i: u32 = 0; i < light_len; i++) {
        var light = light_uniforms[i];

        var light_pos =  camera.view_proj *(world.matrix * vec4<f32>(light.position, 1.0)); 
        var light_dir = light_pos - camera.view_proj * in.world_position;
        var light_mag = dot(normalize(normal), normalize(light_dir));
        // we want distance to be cognizant of world space
        var dist = length(light_dir * vec4(screen_resolution, 1., 1.));
        var attenuation = 1. / (1.0 + light.linear_dropoff * dist + light.quadratic_dropoff * dist * dist);

        var ambient = light.ambient_strength * light.color  * attenuation;
        var diffuse = light.diffuse_strength * light.color  * attenuation * max(light_mag, 0.);
        total_light += ambient + diffuse;
        // total_light += (light.intensity *  light.color / dist );
        // if(dist < 100) {
        //     total_light += light.color;
        // }
        // total_light += vec3(0.3,0.1,0.1);
    }

     // var light1_final = dot(normal, light1_dir) * light1_color / light1_dist;
    // return vec4<f32>(total_light, 1.);
    // return in.world_position / 1000;
    return vec4<f32>(color.xyz * total_light, color.w);
    // return normal;
}
 
 