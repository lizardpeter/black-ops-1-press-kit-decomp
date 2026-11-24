struct VertexInput {
    [[vk::location(0)]] float2 pos : POSITION;
    [[vk::location(1)]] float2 uv : TEXCOORD0;
    [[vk::location(2)]] float2 opacity : TEXCOORD1;
    [[vk::location(3)]] float4 color : COLOR0;
};
struct VertexOutput {
    float4 pos : SV_Position;
    [[vk::location(0)]] float2 uv : TEXCOORD0;
    [[vk::location(1)]] float2 opacity : TEXCOORD1;
    [[vk::location(2)]] float4 color : COLOR0;
};
[[vk::binding(0,0)]] Texture2D<float4> layerTexture;
[[vk::binding(1,0)]] SamplerState layerSampler;
[[vk::binding(2,0)]] Texture2D<float4> secondaryTexture;
float3 encodeSrgb(float3 c) { return select(c <= 0.0031308, c * 12.92, 1.055 * pow(max(c,0.0), 1.0/2.4) - 0.055); }
float3 decodeSrgb(float3 c) { return select(c <= 0.04045, c / 12.92, pow((c+0.055)/1.055, 2.4)); }
[shader("vertex")]
VertexOutput vertexMain(VertexInput v) {
    VertexOutput o;
    o.pos=float4(v.pos,0,1);
    o.uv=v.uv;
    o.opacity=v.opacity;
    o.color=v.color;
    return o;
}
[shader("fragment")]
float4 fragmentMain(VertexOutput v) : SV_Target0 {
    float4 c=layerTexture.Sample(layerSampler,v.uv);
    // The intermediate target stores gamma-encoded RGB. Only the presentation pass writes display-linear values.
    if(v.opacity.y < -1.5) { return float4(v.opacity.x > 0.5 ? decodeSrgb(c.rgb) : c.rgb, 1); }
    if(v.opacity.y>=0) {
        float3 a=encodeSrgb(c.rgb)*v.opacity.x;
        float3 b=encodeSrgb(secondaryTexture.SampleLevel(layerSampler,v.uv,0).rgb);
        return float4(lerp(a,b,v.opacity.y),1);
    }
    c.rgb=encodeSrgb(c.rgb*v.color.rgb);
    c.a*=v.opacity.x*v.color.a;
    return c;
}
