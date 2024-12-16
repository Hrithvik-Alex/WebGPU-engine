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

// fn hash(n: f32) -> f32 { return fract(sin(n)*43758.5453123); }

@fragment
fn fs_main(in: Fragment) -> @location(0) vec4<f32> {

    var c: vec3<f32> =  textureSample(myTexture, mySampler, in.tex_coord).xyz;
	// var u : vec2<f32> = in.Position * 2. - 1.;
	// var c: vec3<f32> = texture(iChannel0, p).xyz;
    var n: vec2<f32> =(in.position_share.xy + vec2<f32>(1.)) / 2;

   var dist = length(n - vec2(0.5));

    var radius = 0.5;
    var softness = 0.02;
    var vignette = smoothstep(radius, 0.2, dist); 
    
    // var time: f32 = timeUniform.time;
	// flicker, grain, vignette, fade in:
	// c += sin(hash(time)) * 0.01;
	// c += hash((hash(n.x) + n.y) * time) * 0.5;
	// c *= smoothstep(length(n * n * n * vec2(0.075, 0.4)), 1.0, 0.4);
    // // c *= smoothstep(0.001, 3.5, time) * 1.5;
	
	// c = dot(c, vec3(0.2126, 0.7152, 0.0722)) 
	//   * vec3(0.2, 1.5 - hash(time) * 0.1,0.4);

    return vec4<f32>(c ,1.);
    // var intensity: f32 = (1.0f / 3.0f) * (color.r + color.g + color.b);
    // var purple: vec3<f32> = intensity * vec3<f32>(176.0 / 255.0, 105.0 / 255.0, 219.0 / 255.0);
    // return vec4<f32>(purple, 1.0) ;

}