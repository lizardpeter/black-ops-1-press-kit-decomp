//! Source-derived PKV1 triangle meshes. Optional asset, no Flash VM or PNG chrome.
use crate::{Layer,Renderer};
use std::collections::BTreeMap;
#[derive(Debug)]struct Batch{rgba:[u8;4],points:Vec<[f32;2]>}
#[derive(Debug)]pub struct ControlPack{entries:BTreeMap<u16,Vec<Batch>>}
#[derive(Clone,Debug,PartialEq)]pub struct ControlVisual{pub y:f32,pub gallery:bool,pub paused:bool,pub progress:f32,pub text:String,pub states:[u8;4]}
impl ControlPack{
 pub fn read(bytes:&[u8])->Result<Self,&'static str>{
  if bytes.len()>2*1024*1024||bytes.get(..4)!=Some(b"PKV1"){return Err("invalid vector pack");}
  let mut cursor=4;fn take<'a>(b:&'a[u8],p:&mut usize,n:usize)->Result<&'a[u8],&'static str>{let end=p.checked_add(n).ok_or("pack overflow")?;let s=b.get(*p..end).ok_or("truncated vector pack")?;*p=end;Ok(s)}
  fn u16(b:&[u8],p:&mut usize)->Result<u16,&'static str>{Ok(u16::from_le_bytes(take(b,p,2)?.try_into().unwrap()))}
  let count=u16(bytes,&mut cursor)?;if count!=19{return Err("unexpected vector entry count");}let mut entries=BTreeMap::new();
  for _ in 0..count{let id=u16(bytes,&mut cursor)?;let n=u16(bytes,&mut cursor)?;if n==0||n>32{return Err("invalid batch count");}let mut batches=vec![];
   for _ in 0..n{let rgba=take(bytes,&mut cursor,4)?.try_into().unwrap();let n=u32::from_le_bytes(take(bytes,&mut cursor,4)?.try_into().unwrap()) as usize;if n==0||n%3!=0||n>100000{return Err("invalid vertex count");}
    let raw=take(bytes,&mut cursor,n*8)?;let mut points=Vec::with_capacity(n);for p in raw.chunks_exact(8){let x=f32::from_le_bytes(p[..4].try_into().unwrap());let y=f32::from_le_bytes(p[4..].try_into().unwrap());if !x.is_finite()||!y.is_finite()||x.abs()>4096.||y.abs()>4096.{return Err("invalid mesh coordinate");}points.push([x,y]);}batches.push(Batch{rgba,points});
   }if entries.insert(id,batches).is_some(){return Err("duplicate vector entry");}
  }
  if cursor!=bytes.len(){return Err("trailing pack data");}for id in [25,160,161,165,167,170,172,174,179,223,1567,1568,1569,1693,1694,1695,1714,1715,1716]{if !entries.contains_key(&id){return Err("missing control state");}}
  Ok(Self{entries})
 }
 fn append(&self,renderer:&Renderer,device:&wgpu::Device,queue:&wgpu::Queue,layers:&mut Vec<Layer>,id:u16,x:f32,y:f32){
  if let Some(batches)=self.entries.get(&id){for b in batches{let vertices:Vec<_>=b.points.iter().map(|p|[x+p[0],y+p[1]]).collect();if let Some(l)=renderer.mesh(device,queue,b.rgba,&vertices){layers.push(l);}}}
 }
 pub fn layers(&self,r:&Renderer,d:&wgpu::Device,q:&wgpu::Queue,v:&ControlVisual)->Vec<Layer>{
  let mut out=vec![];let y=v.y;let mut add=|id,x|self.append(r,d,q,&mut out,id,x,y);
  if v.gallery{add(223,200.);add(1693+u16::from(v.states[2].min(2)),215.);add(1714+u16::from(v.states[3].min(2)),365.);}
  else{add(160,200.);add(25,200.);let ids=if v.paused{[170,172,174]}else{[161,165,167]};add(ids[v.states[0].min(2) as usize],215.);}
  add(1567+u16::from(v.states[1].min(2)),961.);
  self.append(r,d,q,&mut out,179,204.,y+48.);
  if !v.gallery&&v.progress>0.{out.push(r.layer(d,q,&[242,180,7,255],1,1,[204.,y+48.,872.*v.progress.clamp(0.,1.),10.]));}
  if let Some(l)=r.counter(d,q,&v.text,y,v.gallery){out.push(l);}out
 }
}
#[derive(Default)]pub struct ControlCache{key:Option<ControlVisual>,layers:Vec<Layer>}
impl ControlCache{
 pub fn update(&mut self,pack:Option<&ControlPack>,r:&Renderer,d:&wgpu::Device,q:&wgpu::Queue,mut v:ControlVisual){
  v.progress=(v.progress.clamp(0.,1.)*872.).round()/872.;
  if self.key.as_ref()==Some(&v){return;}
  self.layers=if let Some(p)=pack{p.layers(r,d,q,&v)}else if v.gallery{r.gallery_vectors(d,q,v.y,&v.text)}else{let mut l=r.video_vectors(d,q,v.y,v.paused,v.progress);if let Some(t)=r.counter(d,q,&v.text,v.y,false){l.push(t);}l};self.key=Some(v);
 }
 pub fn layers(&self)->&[Layer]{&self.layers}
 pub fn clear(&mut self){self.key=None;self.layers.clear();}
}
#[cfg(test)]mod tests{use super::*;#[test]fn rejects_invalid_packs(){for p in [b"".as_slice(),b"PKV1",b"PKV1\x13\0",b"PKV1\0\0"]{assert!(ControlPack::read(p).is_err());}}}
