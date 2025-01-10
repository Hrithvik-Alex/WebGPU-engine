//#include uniform.wgsl
//#include texture.wgsl

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(3) extra_info: u32,
};

struct VertexOutput {
    @invariant @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) extra_info: u32,
};



@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position =  camera.view_proj * /*camera.view *  */  (world.matrix * vec4<f32>(model.position, 1.0));
    out.tex_coords = model.tex_coords;
    out.extra_info = model.extra_info;
    out.world_position =(world.matrix * vec4<f32>(model.position, 1.0)); 
    return out;
}



@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec4<f32>;
    var texture_num = in.extra_info & 255;
    switch texture_num {
        case 0u: {
            color = textureSample(t_character, pixel_sampler, in.tex_coords);
        }

        case 1u: {
            color = textureSample(t_minotaur, pixel_sampler, in.tex_coords);
        }

        case 2u: {
            // color = textureSample(t_bg, pixel_sampler, in.tex_coords);
            color = vec4(0.1,0.1,0.1,1);
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
    return vec4(0,0,1.,1.);
}