//! Original trailer and screenshot button states, plus native menu glyphs including wallpapers.
use crate::{Layer, Renderer};
#[derive(Clone, Copy)]
struct Instance { id:u8, x:f32, y:f32 }
struct Batch { rgba:[u8;4], vertices:Vec<[f32;2]> }
pub struct MenuPack { instances:[Vec<Instance>;2], frames:[u8;105], states:[Vec<Batch>;6],labels:[Vec<Batch>;3] }
impl MenuPack {
 pub fn read(bytes:&[u8])->Result<Self,&'static str>{
  let v3=bytes.get(..4)==Some(b"PKM3");let v2=v3||bytes.get(..4)==Some(b"PKM2");if (!v2&&bytes.get(..4)!=Some(b"PKM1"))||bytes.len()>65536{return Err("invalid menu vector pack");}
  let mut pos=4;
  fn take<const N:usize>(b:&[u8],p:&mut usize)->Result<[u8;N],&'static str>{let end=p.checked_add(N).ok_or("menu overflow")?;let s=b.get(*p..end).ok_or("truncated menu")?;*p=end;Ok(s.try_into().unwrap())}
  let mut instances:[Vec<Instance>;2]=std::array::from_fn(|_|vec![]);
  for (g,ids) in [&[1u8,2,4,5,6][..],&(1u8..=20).collect::<Vec<_>>()[..]].into_iter().enumerate(){
   if usize::from(take::<1>(bytes,&mut pos)?[0])!=ids.len(){return Err("invalid menu instance count");}
   for &id in ids {let v=take::<1>(bytes,&mut pos)?[0];let x=i32::from_le_bytes(take(bytes,&mut pos)?);let y=i32::from_le_bytes(take(bytes,&mut pos)?);if v!=id||!(0..=10000).contains(&x)||!(0..=10000).contains(&y){return Err("invalid menu placement");}instances[g].push(Instance{id,x:(10537+x)as f32/20.,y:(860+y)as f32/20.});}
  }
  let frames=take::<105>(bytes,&mut pos)?;
  for (i,&f) in frames.iter().enumerate(){let n=i+1;let expected=if [87,89,91,92,93,94].contains(&n){1}else if [97,99,101,102,103,104].contains(&n){2}else if v3&&[77,79,81,82,83,84].contains(&n){3}else{0};if f!=expected{return Err("unexpected menu frame");}}
  let mut states:[Vec<Batch>;6]=std::array::from_fn(|_|vec![]);
  for batches in &mut states{let n=take::<1>(bytes,&mut pos)?[0];if !(1..=4).contains(&n){return Err("invalid menu batch count");}
   for _ in 0..n{let rgba=take(bytes,&mut pos)?;let n=u16::from_le_bytes(take(bytes,&mut pos)?)as usize;if n==0||n%3!=0||n>2048{return Err("invalid menu vertices");}let mut vertices=vec![];for _ in 0..n{let x=f32::from_le_bytes(take(bytes,&mut pos)?);let y=f32::from_le_bytes(take(bytes,&mut pos)?);if !x.is_finite()||!y.is_finite()||x.abs()>512.||y.abs()>512.{return Err("invalid menu vertex");}vertices.push([x,y]);}batches.push(Batch{rgba,vertices});}
  }
  let mut labels:[Vec<Batch>;3]=std::array::from_fn(|_|vec![]);
  if v2{
   let count=u16::from_le_bytes(take(bytes,&mut pos)?)as usize;
   if !(1..=256).contains(&count){return Err("invalid glyph count");}
   let mut glyphs=Vec::with_capacity(count);let mut dictionary_vertices=0usize;
   for _ in 0..count{
    let n=u16::from_le_bytes(take(bytes,&mut pos)?)as usize;
    if !(1..=4096).contains(&n){return Err("invalid glyph size");}
    let end=pos.checked_add(n).ok_or("glyph overflow")?;let raw=bytes.get(pos..end).ok_or("truncated glyph")?;pos=end;
    let mesh=crate::font::glyph_mesh(raw).ok_or("invalid glyph contour")?;dictionary_vertices+=mesh.len();
    if mesh.len()>100_000||dictionary_vertices>500_000{return Err("glyph mesh too large");}
    glyphs.push(mesh);
   }
   for group in labels.iter_mut().take(if v3{3}else{2}){
    let count=u16::from_le_bytes(take(bytes,&mut pos)?)as usize;
    if !(1..=512).contains(&count){return Err("invalid text count");}
    let mut batches=std::collections::BTreeMap::<[u8;4],Vec<[f32;2]>>::new();let mut vertices=0usize;
    for _ in 0..count{
     let index=u16::from_le_bytes(take(bytes,&mut pos)?)as usize;let rgba=take(bytes,&mut pos)?;let mut m=[0f64;6];
     for x in &mut m{*x=f32::from_le_bytes(take(bytes,&mut pos)?)as f64;}
     if m.iter().any(|v|!v.is_finite()||v.abs()>4096.)||m[..4].iter().any(|v|v.abs()>1.){return Err("invalid glyph transform");}
     let mesh=glyphs.get(index).ok_or("invalid glyph index")?;vertices+=mesh.len();if vertices>1_000_000{return Err("label mesh budget exceeded");}
     let [sx,sy,rx,ry,tx,ty]=m;
     batches.entry(rgba).or_default().extend(mesh.iter().map(|p|[(sx*p[0]+rx*p[1]+tx)as f32,(ry*p[0]+sy*p[1]+ty)as f32]));
    }
    *group=batches.into_iter().map(|(rgba,vertices)|Batch{rgba,vertices}).collect();
   }
  }
  if pos!=bytes.len(){return Err("trailing menu bytes");}Ok(Self{instances,frames,states,labels})
 }
 pub fn label_layers(&self,r:&Renderer,d:&wgpu::Device,q:&wgpu::Queue,group:u8)->Vec<Layer>{
  if !(1..=3).contains(&group){return vec![];}
  self.label_layers_shifted(r,d,q,group,0.)
 }
 fn label_layers_shifted(&self,r:&Renderer,d:&wgpu::Device,q:&wgpu::Queue,group:u8,dx:f32)->Vec<Layer>{
  if !(1..=3).contains(&group){return vec![];}
  self.labels[usize::from(group-1)].iter().filter_map(|b|{if dx==0.{r.mesh(d,q,b.rgba,&b.vertices)}else{let v:Vec<_>=b.vertices.iter().map(|p|[p[0]+dx,p[1]]).collect();r.mesh(d,q,b.rgba,&v)}}).collect()
 }
 pub fn group(&self,frame:u16)->u8{self.frames[usize::from(frame.clamp(1,105)-1)]}
 pub fn layers(&self,r:&Renderer,d:&wgpu::Device,q:&wgpu::Queue,group:u8,selected:u8,pressed:bool)->Vec<Layer>{
  if !(1..=2).contains(&group){return vec![];}let g=usize::from(group-1);let mut batches:std::collections::BTreeMap<(usize,[u8;4]),Vec<[f32;2]>>=std::collections::BTreeMap::new();
  for instance in &self.instances[g]{let state=if instance.id==selected{if pressed{2}else{1}}else{0};for (order,b) in self.states[g*3+state].iter().enumerate(){batches.entry((order,b.rgba)).or_default().extend(b.vertices.iter().map(|p|[p[0]+instance.x,p[1]+instance.y]));}}
  batches.into_iter().filter_map(|((_,rgba),v)|r.mesh(d,q,rgba,&v)).collect()
 }
}
#[derive(Default)]pub struct MenuCache{key:Option<(u8,u8,bool,bool)>,layers:Vec<Layer>,labels:Vec<Layer>}
impl MenuCache{
 pub fn update(&mut self,p:&MenuPack,r:&Renderer,d:&wgpu::Device,q:&wgpu::Queue,frame:u16,selected:u8,pressed:bool){let group=p.group(frame);let shifted=group==3&&(82..=84).contains(&frame);let key=(group,selected,pressed,shifted);if self.key==Some(key){return;}if self.key.map(|k|(k.0,k.3))!=Some((group,shifted)){self.labels=p.label_layers_shifted(r,d,q,group,if shifted{-0.85}else{0.});}self.layers=p.layers(r,d,q,group,selected,pressed);self.key=Some(key);}
 pub fn labels(&self)->&[Layer]{&self.labels}
 pub fn layers(&self)->&[Layer]{&self.layers}
}
#[cfg(test)]mod tests{use super::*;#[test]fn rejects_truncated(){for b in [b"".as_slice(),b"PKM1",b"PKM1\x05"]{assert!(MenuPack::read(b).is_err());}}}
