@group(0) @binding(0) var ds_texture: texture_2d<u32>;
@group(0) @binding(1) var<storage, read_write> buffer_w: array<u32>;

const wkgp_size_x : u32 = 8;
const wkgp_size_y : u32 = 8;
const wkgp_size_z : u32 = 1;

@compute @workgroup_size(wkgp_size_x,wkgp_size_y,wkgp_size_z) fn convert_depth_to_color(
    @builtin(workgroup_id) workgroup_id : vec3<u32>,
    @builtin(local_invocation_id) local_invocation_id : vec3<u32>,

    @builtin(local_invocation_index) local_invocation_index: u32,
    @builtin(num_workgroups) num_workgroups: vec3<u32>


) {

    let texture_dim = textureDimensions(ds_texture);
    let global_x = (workgroup_id.x * wkgp_size_x + local_invocation_id.x);
    let global_y = (workgroup_id.y * wkgp_size_y + local_invocation_id.y);
    // let global_z = (workgroup_id.z * wkgp_size_z + local_invocation_id.z);

    if (global_x >= texture_dim.x || global_y >= texture_dim.y) {
        return;
    }

    let write_index = global_x + global_y * texture_dim.x;
    let stencil_value = textureLoad(ds_texture, vec2(global_x, global_y), 0).r;


    let strength = stencil_value * 64;
    let value : u32 = 0xff000000 | (strength << 16) | (strength << 8) | (strength);
    buffer_w[write_index] = value;

}