struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct SpriteUniform {
    position: vec2<f32>,
    velocity: vec2<f32>,
    _padding: vec2<f32>,
    color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> sprite: SpriteUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position + sprite.position, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return sprite.color;
}
