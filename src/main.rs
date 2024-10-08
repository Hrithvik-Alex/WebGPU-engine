use WebGPU_engine::run;

// TODO: why cant main be async in rust?
fn main() {
    pollster::block_on(run());
}
