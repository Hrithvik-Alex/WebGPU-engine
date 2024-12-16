struct CameraUniform {
    view_proj: mat4x4<f32>,
};

struct WorldUniform {
    matrix: mat4x4<f32>,
};

// struct ProjectionUniform {
//     proj: mat4x4<f32>,
// };

struct VertexOut {
  @builtin(position) position: vec4f,
};

@group(0) @binding(0) 
var<uniform> camera: CameraUniform;

@group(0) @binding(1) 
var<uniform> world: WorldUniform;

@group(1) @binding(0)
var<storage, read> positions: array<f32>;

@group(1) @binding(1)
var<storage, read> indices: array<u32>;

@vertex fn vs_main(@builtin(vertex_index) vNdx: u32) -> VertexOut {
  // indices make a triangle so for every 3 indices we need to output
  // 6 values
  let triNdx = vNdx / 6;
  // 0 1 0 1 0 1  0 1 0 1 0 1  vNdx % 2
  // 0 0 1 1 2 2  3 3 4 4 5 5  vNdx / 2
  // 0 1 1 2 2 3  3 4 4 5 5 6  vNdx % 2 + vNdx / 2
  // 0 1 1 2 2 0  0 1 1 2 2 0  (vNdx % 2 + vNdx / 2) % 3
  let vertNdx = (vNdx % 2 + vNdx / 2) % 3;
  let index = indices[triNdx * 3 + vertNdx];


  let pNdx = index * 8; // modelvertex2d stride length
  let position = vec4f(positions[pNdx], positions[pNdx + 1], 0.2, 1.0);


  var vOut: VertexOut;
  vOut.position = camera.view_proj * ( world.matrix * position );
  return vOut;
}

@fragment fn fs_main() -> @location(0) vec4f {
  return vec4f(1.);
}