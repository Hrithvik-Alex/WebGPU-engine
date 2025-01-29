//#include uniform.wgsl
//#include texture.wgsl
//#include model_vertex.wgsl

// struct VertexInput {
//     @location(0) position: vec3<f32>,
//     @location(1) tex_coords: vec2<f32>,
//     @location(3) extra_info: u32,
// };

// struct VertexOutput {
//     @invariant @builtin(position) clip_position: vec4<f32>,
//     @location(0) world_position: vec4<f32>,
//     @location(1) tex_coords: vec2<f32>,
//     @location(2) extra_info: u32,
// };



@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position =  camera.screen_to_clip * /*camera.view *  */  (world.world_to_screen * vec4<f32>(model.position, 1.0));
    out.tex_coords = model.tex_coords;
    out.extra_info = model.extra_info;
    out.world_position =(world.world_to_screen * vec4<f32>(model.position, 1.0)); 
    return out;
}



@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var t_info = get_texture_color(in.extra_info, in.tex_coords);
    var color = t_info.color;

   if (color.w == 0 ) {
    discard;
   }

   return color;
}