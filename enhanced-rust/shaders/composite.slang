struct VertexInput { [[vk::location(0)]] float2 pos : POSITION; [[vk::location(1)]] float2 uv : TEXCOORD0; };
struct VertexOutput { float4 pos : SV_Position; [[vk::location(0)]] float2 uv : TEXCOORD0; };
[[vk::binding(0,0)]] Texture2D<float4> layerTexture;
[[vk::binding(1,0)]] SamplerState layerSampler;
[shader("vertex")]
VertexOutput vertexMain(VertexInput v) { VertexOutput o; o.pos=float4(v.pos,0,1); o.uv=v.uv; return o; }
[shader("fragment")]
float4 fragmentMain(VertexOutput v) : SV_Target0 { return layerTexture.Sample(layerSampler,v.uv); }
