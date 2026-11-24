// Slang counterpart of the portable filter.wgsl; regenerate/check with offline slangc.
struct VertexInput {
 [[vk::location(0)]] float2 pos : POSITION;
 [[vk::location(2)]] float2 uv : TEXCOORD0;
 [[vk::location(1)]] float2 opacity : TEXCOORD1;
 [[vk::location(3)]] float4 color : COLOR0;
};
struct VertexOutput {
 float4 pos : SV_Position;
 [[vk::location(0)]] float2 uv : TEXCOORD0;
 [[vk::location(1)]] float2 opacity : TEXCOORD1;
 [[vk::location(2)]] float4 color : COLOR0;
};
struct GroupScale { float4 scale; };
[[vk::binding(0,1)]] ConstantBuffer<GroupScale> groupScale;
[[vk::binding(0,0)]] Texture2D<float4> sourceTexture;
[[vk::binding(1,0)]] SamplerState sourceSampler;
[shader("vertex")] VertexOutput groupVertex(VertexInput v) {
 VertexOutput o; o.pos=float4(v.pos*groupScale.scale.xy,0,1);
 o.uv=v.uv; o.opacity=v.opacity; o.color=v.color; return o;
}
[shader("fragment")] float4 blurFragment(VertexOutput v):SV_Target0 {
 float2 uv=0.5+(v.uv-0.5)*v.color.xy;
 float halfWidth=min(v.opacity.y*0.5,127.0);
 if(halfWidth<=0.5)return sourceTexture.SampleLevel(sourceSampler,uv,0)*v.opacity.x;
 uint width,height;sourceTexture.GetDimensions(width,height);
 int limit=int(ceil(halfWidth+0.5));float4 sum=0;float weight=0;
 for(int k=-limit;k<=limit;k++){
  float d=float(k),w=max(0.0,min(d+0.5,halfWidth)-max(d-0.5,-halfWidth));
  if(w>0){sum+=sourceTexture.SampleLevel(sourceSampler,uv+float2(0,d/float(height)),0)*w;weight+=w;}
 }
 return sum*(v.opacity.x/max(weight,0.0001));
}
