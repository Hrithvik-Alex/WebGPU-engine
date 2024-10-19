struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0)  color: vec3<f32>,
    @location(1) vert_pos: vec3<f32>,
};

@vertex
fn vs_main_regular(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}
 
@fragment
fn fs_main_regular(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
 
 @vertex
fn vs_main_challenge(
   model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.vert_pos = out.clip_position.xyz;
    return out;
}
 
@fragment
fn fs_main_challenge(in: VertexOutput) -> @location(0)  vec4<f32> {
    return vec4<f32>(in.vert_pos.x + 0.5, in.vert_pos.y + 0.5, -in.vert_pos.x - in.vert_pos.y, 1.0);
}