// Native one-pass source vertical blur; transparent group target stores premultiplied gamma RGB.
struct VertexInput {
 @location(0) pos: vec2<f32>,
 @location(2) uv: vec2<f32>,
 @location(1) opacity: vec2<f32>,
 @location(3) color: vec4<f32>,
};
struct VertexOutput {
 @builtin(position) pos: vec4<f32>,
 @location(0) uv: vec2<f32>,
 @location(1) opacity: vec2<f32>,
 @location(2) color: vec4<f32>,
};
struct GroupScale { scale: vec4<f32> };
@group(1) @binding(0) var<uniform> groupScale: GroupScale;
@vertex fn groupVertex(v: VertexInput)->VertexOutput {
 var o: VertexOutput;
 o.pos=vec4<f32>(v.pos*groupScale.scale.xy,0.0,1.0);
 o.uv=v.uv; o.opacity=v.opacity; o.color=v.color; return o;
}
@group(0) @binding(0) var sourceTexture: texture_2d<f32>;
@group(0) @binding(1) var sourceSampler: sampler;
@fragment fn blurFragment(v: VertexOutput)->@location(0) vec4<f32> {
 let uv=vec2<f32>(0.5)+(v.uv-vec2<f32>(0.5))*v.color.xy;
 let halfWidth=min(v.opacity.y*0.5,127.0);
 if halfWidth<=0.5 { return textureSampleLevel(sourceTexture,sourceSampler,uv,0.0)*v.opacity.x; }
 let h=f32(textureDimensions(sourceTexture).y);
 let limit=i32(ceil(halfWidth+0.5));
 var sum=vec4<f32>(0.0);
 var weight=0.0;
 for(var k:i32=-limit;k<=limit;k=k+1){
  let d=f32(k);
  let w=max(0.0,min(d+0.5,halfWidth)-max(d-0.5,-halfWidth));
  if w>0.0 { sum+=textureSampleLevel(sourceTexture,sourceSampler,uv+vec2<f32>(0.0,d/h),0.0)*w; weight+=w; }
 }
 return sum*(v.opacity.x/max(weight,0.0001));
}
