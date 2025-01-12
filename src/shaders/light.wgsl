
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
@group(2) @binding(2)
var<uniform> ambient_light_intensity: f32;

fn calc_light_forward(in_color: vec4<f32>, normal: vec4<f32>, world_position: vec4<f32>) -> vec4<f32> {
    var total_light = vec3(ambient_light_intensity);


    for(var i: u32 = 0; i < light_len; i++) {
        var light = light_uniforms[i];

        var light_pos =  camera.screen_to_clip *(world.world_to_screen * vec4<f32>(light.position, 1.0)); 
        var light_dir = light_pos - camera.screen_to_clip * world_position;
        var light_mag = dot(normalize(normal), normalize(light_dir));
        // we want distance to be cognizant of world space
        var dist = length(world.screen_to_world * camera.clip_to_screen * light_dir);
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
    return vec4<f32>(in_color.xyz * total_light, in_color.w);
}