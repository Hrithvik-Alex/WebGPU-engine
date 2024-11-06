// language: metal1.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct VertexInput {
    metal::float3 position;
    metal::float3 color;
};
struct VertexOutput {
    metal::float4 clip_position;
    metal::float3 color;
    metal::float3 vert_pos;
};

struct vs_main_regularInput {
    metal::float3 position [[attribute(0)]];
    metal::float3 color [[attribute(1)]];
};
struct vs_main_regularOutput {
    metal::float4 clip_position [[position]];
    metal::float3 color [[user(loc0), center_perspective]];
    metal::float3 vert_pos [[user(loc1), center_perspective]];
};
vertex vs_main_regularOutput vs_main_regular(
  vs_main_regularInput varyings [[stage_in]]
) {
    const VertexInput model = { varyings.position, varyings.color };
    VertexOutput out = {};
    out.color = model.color;
    out.clip_position = metal::float4(model.position, 1.0);
    VertexOutput _e8 = out;
    const auto _tmp = _e8;
    return vs_main_regularOutput { _tmp.clip_position, _tmp.color, _tmp.vert_pos };
}


struct fs_main_regularInput {
    metal::float3 color [[user(loc0), center_perspective]];
    metal::float3 vert_pos [[user(loc1), center_perspective]];
};
struct fs_main_regularOutput {
    metal::float4 member_1 [[color(0)]];
};
fragment fs_main_regularOutput fs_main_regular(
  fs_main_regularInput varyings_1 [[stage_in]]
, metal::float4 clip_position [[position]]
) {
    const VertexOutput in = { clip_position, varyings_1.color, varyings_1.vert_pos };
    return fs_main_regularOutput { metal::float4(in.color, 1.0) };
}


struct vs_main_challengeInput {
    metal::float3 position [[attribute(0)]];
    metal::float3 color [[attribute(1)]];
};
struct vs_main_challengeOutput {
    metal::float4 clip_position [[position]];
    metal::float3 color [[user(loc0), center_perspective]];
    metal::float3 vert_pos [[user(loc1), center_perspective]];
};
vertex vs_main_challengeOutput vs_main_challenge(
  vs_main_challengeInput varyings_2 [[stage_in]]
) {
    const VertexInput model_1 = { varyings_2.position, varyings_2.color };
    VertexOutput out_1 = {};
    out_1.color = model_1.color;
    out_1.clip_position = metal::float4(model_1.position, 1.0);
    metal::float4 _e10 = out_1.clip_position;
    out_1.vert_pos = _e10.xyz;
    VertexOutput _e12 = out_1;
    const auto _tmp = _e12;
    return vs_main_challengeOutput { _tmp.clip_position, _tmp.color, _tmp.vert_pos };
}


struct fs_main_challengeInput {
    metal::float3 color [[user(loc0), center_perspective]];
    metal::float3 vert_pos [[user(loc1), center_perspective]];
};
struct fs_main_challengeOutput {
    metal::float4 member_3 [[color(0)]];
};
fragment fs_main_challengeOutput fs_main_challenge(
  fs_main_challengeInput varyings_3 [[stage_in]]
, metal::float4 clip_position_1 [[position]]
) {
    const VertexOutput in_1 = { clip_position_1, varyings_3.color, varyings_3.vert_pos };
    return fs_main_challengeOutput { metal::float4(in_1.vert_pos.x + 0.5, in_1.vert_pos.y + 0.5, -(in_1.vert_pos.x) - in_1.vert_pos.y, 1.0) };
}
