@binding(0) @group(0) var layerTexture_0 : texture_2d<f32>;

@binding(1) @group(0) var layerSampler_0 : sampler;

@binding(2) @group(0) var secondaryTexture_0 : texture_2d<f32>;

fn encodeSrgb_0( c_0 : vec3<f32>) -> vec3<f32>
{
    return select(vec3<f32>(1.0549999475479126f) * pow(max(c_0, vec3<f32>(0.0f)), vec3<f32>(0.4166666567325592f)) - vec3<f32>(0.054999999701976776f), c_0 * vec3<f32>(12.920000076293945f), (c_0 <= vec3<f32>(0.0031308000907301903f)));
}

fn decodeSrgb_0( c_1 : vec3<f32>) -> vec3<f32>
{
    return select(pow((c_1 + vec3<f32>(0.054999999701976776f)) / vec3<f32>(1.0549999475479126f), vec3<f32>(2.4000000953674316f)), c_1 / vec3<f32>(12.920000076293945f), (c_1 <= vec3<f32>(0.040449999272823334f)));
}

struct pixelOutput_0
{
    @location(0) output_0 : vec4<f32>,
};

struct pixelInput_0
{
    @location(0) uv_0 : vec2<f32>,
    @location(1) opacity_0 : vec2<f32>,
};

@fragment
fn fragmentMain( _S1 : pixelInput_0, @builtin(position) pos_0 : vec4<f32>) -> pixelOutput_0
{
    var c_2 : vec4<f32> = (textureSample((layerTexture_0), (layerSampler_0), (_S1.uv_0)));
    var _S2 : f32 = _S1.opacity_0.y;
    if((_S2 >= 0.0f))
    {
        var _S3 : pixelOutput_0 = pixelOutput_0( vec4<f32>(decodeSrgb_0(mix(encodeSrgb_0(c_2.xyz) * vec3<f32>(_S1.opacity_0.x), encodeSrgb_0((textureSampleLevel((secondaryTexture_0), (layerSampler_0), (_S1.uv_0), (0.0f))).xyz), vec3<f32>(_S2))), 1.0f) );
        return _S3;
    }
    c_2[i32(3)] = c_2[i32(3)] * _S1.opacity_0.x;
    var _S4 : pixelOutput_0 = pixelOutput_0( c_2 );
    return _S4;
}

