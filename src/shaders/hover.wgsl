//#include uniform.wgsl
//#include texture.wgsl
//#include model_vertex.wgsl
//#include light.wgsl



@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var offset = cos(time * 5.);
    var in = model;
    in.position.y += offset * 20.;

    return vertex_in_to_out(in);

}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {


    var t_info = get_texture_color(in.extra_info, in.tex_coords);
    var color = t_info.color;
    var normal = t_info.normal;

   if (color.w == 0 ) {
    discard;
   }

   return calc_light_forward(color, normal, in.world_position);

}