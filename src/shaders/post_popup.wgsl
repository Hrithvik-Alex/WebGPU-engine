//#include post.wgsl


@fragment
fn fs_main(in: Fragment) -> @location(0) vec4<f32> {

    var c: vec3<f32> =  textureSample(myTexture, mySampler, in.tex_coord).xyz;



    return vec4(c * 0.025,1.);
}