@group(0) @binding(0) var<storage, read> buffer_r: array<u32>;
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

    let original_x = (workgroup_id.x * wkgp_size_x + local_invocation_id.x);
    let global_x = original_x / 4; // stencil buffer is packed 8 bits each
    let global_y = (workgroup_id.y * wkgp_size_y + local_invocation_id.y);
    let global_z = (workgroup_id.z * wkgp_size_z + local_invocation_id.z);
    
    // let read_index = global_x 
    // + global_y * (num_workgroups.x * wkgp_size_x / 4)
    //  + global_z * (num_workgroups.x * num_workgroups.y * wkgp_size_x * wkgp_size_y);

    let local_index = local_invocation_id.x / 4 + local_invocation_id.y * wkgp_size_x / 4 + local_invocation_id.z * wkgp_size_x * wkgp_size_y / 4;


    // let local_invocation_index = local_invocation_id.x / 4 +
    //   local_invocation_id.y * (num_workgroups.x * 16) + 
    //   local_invocation_id.z * (num_workgroups.x * 16) * (num_workgroups.y * 16);
    
     
    let workgroup_index =  
        workgroup_id.x +
        workgroup_id.y * num_workgroups.x +
        workgroup_id.z * num_workgroups.x * num_workgroups.y;

    let read_index =workgroup_index * (wkgp_size_x * wkgp_size_y * wkgp_size_z / 4) + (local_index);

    let write_index = workgroup_index * (wkgp_size_x * wkgp_size_y * wkgp_size_z) + local_invocation_index;



    // let stencil_value : u32=  buffer_r[read_index]& 0xff;
    let stencil_value = (buffer_r[read_index] >> (8 * (3 - ( original_x % 4 )))) & 0xff;
    let strength = stencil_value * 64;
    let value : u32 = 0xff000000 | (strength << 16) | (strength << 8) | (strength);
    buffer_w[write_index] = value;

}