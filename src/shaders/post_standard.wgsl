//#include post.wgsl

// fn hash(n: f32) -> f32 { return fract(sin(n)*43758.5453123); }

@fragment
fn fs_main(in: Fragment) -> @location(0) vec4<f32> {

    var c: vec3<f32> =  textureSample(myTexture, mySampler, in.tex_coord).xyz;

    var n: vec2<f32> =(in.position_share.xy + vec2<f32>(1.)) / 2;

   var dist = length(n - vec2(0.5));

    var radius = 0.9;
    var vignette = smoothstep(0.2,1., dist / radius); 
    

    return vec4(c,1.);
}