struct VertexOutput_0
{
    @builtin(position) pos_0 : vec4<f32>,
    @location(0) uv_0 : vec2<f32>,
    @location(1) opacity_0 : vec2<f32>,
};

struct vertexInput_0
{
    @location(0) pos_1 : vec2<f32>,
    @location(2) uv_1 : vec2<f32>,
    @location(1) opacity_1 : vec2<f32>,
};

@vertex
fn vertexMain( _S1 : vertexInput_0) -> VertexOutput_0
{
    var o_0 : VertexOutput_0;
    o_0.pos_0 = vec4<f32>(_S1.pos_1, 0.0f, 1.0f);
    o_0.uv_0 = _S1.uv_1;
    o_0.opacity_0 = _S1.opacity_1;
    return o_0;
}

