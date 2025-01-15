struct TimeUniform {
    time: f32,
};

@group(0) @binding(0) var myTexture: texture_2d<f32>;
@group(0) @binding(1) var mySampler: sampler;
@group(0) @binding(2) var<uniform> timeUniform: TimeUniform;

struct Fragment {
    @builtin(position) position : vec4<f32>,
    @location(0) tex_coord : vec2<f32>,
    @location(1) position_share : vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) VertexIndex: u32) -> Fragment {

    var positions = array<vec2<f32>, 6>(
        vec2<f32>( -1.0,  1.0),
        vec2<f32>( -1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0,  1.0),
    );

    var output : Fragment;

    var pos: vec2<f32> = positions[VertexIndex];
    output.position = vec4<f32>(pos, 0.0, 1.0);
    var tex: vec2<f32> = (pos/2.0 + vec2(0.5));
    output.tex_coord = vec2(tex.x, 1 - tex.y);
    output.position_share =pos;
    return output;
}