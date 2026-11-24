@binding(0) @group(0) var layerTexture_0 : texture_2d<f32>;

@binding(1) @group(0) var layerSampler_0 : sampler;

struct pixelOutput_0
{
    @location(0) output_0 : vec4<f32>,
};

struct pixelInput_0
{
    @location(0) uv_0 : vec2<f32>,
};

@fragment
fn fragmentMain( _S1 : pixelInput_0, @builtin(position) pos_0 : vec4<f32>) -> pixelOutput_0
{
    var _S2 : pixelOutput_0 = pixelOutput_0( (textureSample((layerTexture_0), (layerSampler_0), (_S1.uv_0))) );
    return _S2;
}

