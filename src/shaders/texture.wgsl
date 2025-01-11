@group(1) @binding(0)
var pixel_sampler: sampler;

@group(1) @binding(1)
var t_character: texture_2d<f32>;
@group(1) @binding(2)
var n_character: texture_2d<f32>;

@group(1) @binding(3)
var t_minotaur: texture_2d<f32>;
// @group(2) @binding(4)
// var n_minotaur: texture_2d<f32>;

@group(1) @binding(4)
var t_bg1: texture_2d<f32>;

@group(1) @binding(5)
var t_bg2: texture_2d<f32>;

@group(1) @binding(6)
var t_bg3: texture_2d<f32>;

@group(1) @binding(7)
var t_bg4: texture_2d<f32>;