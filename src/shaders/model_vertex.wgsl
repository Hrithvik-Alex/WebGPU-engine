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




fn vertex_in_to_out( model: VertexInput ) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.normal_coords = model.normal_coords;
    out.clip_position =  camera.screen_to_clip * /*camera.view *  */  (world.world_to_screen * vec4<f32>(model.position, 1.0));
    out.world_position =(world.world_to_screen * vec4<f32>(model.position, 1.0)); 
    out.extra_info = model.extra_info;
    return out;
} 