//! Original 5 trailer and 20 screenshot button instances, with three vector states.
use crate::{Layer, Renderer};
#[derive(Clone, Copy)]
struct Instance { id:u8, x:f32, y:f32 }
struct Batch { rgba:[u8;4], vertices:Vec<[f32;2]> }
pub struct MenuPack { instances:[Vec<Instance>;2], frames:[u8;105], states:[Vec<Batch>;6] }
impl MenuPack {
 pub fn read(bytes:&[u8])->Result<Self,&'static str>{
  if bytes.get(..4)!=Some(b"PKM1")||bytes.len()>65536{return Err("invalid menu vector pack");}
  let mut pos=4;
  fn take<const N:usize>(b:&[u8],p:&mut usize)->Result<[u8;N],&'static str>{let end=p.checked_add(N).ok_or("menu overflow")?;let s=b.get(*p..end).ok_or("truncated menu")?;*p=end;Ok(s.try_into().unwrap())}
  let mut instances:[Vec<Instance>;2]=std::array::from_fn(|_|vec![]);
  for (g,ids) in [&[1u8,2,4,5,6][..],&(1u8..=20).collect::<Vec<_>>()[..]].into_iter().enumerate(){
   if usize::from(take::<1>(bytes,&mut pos)?[0])!=ids.len(){return Err("invalid menu instance count");}
   for &id in ids {let v=take::<1>(bytes,&mut pos)?[0];let x=i32::from_le_bytes(take(bytes,&mut pos)?);let y=i32::from_le_bytes(take(bytes,&mut pos)?);if v!=id||!(0..=10000).contains(&x)||!(0..=10000).contains(&y){return Err("invalid menu placement");}instances[g].push(Instance{id,x:(10537+x)as f32/20.,y:(860+y)as f32/20.});}
  }
  let frames=take::<105>(bytes,&mut pos)?;
  for (i,&f) in frames.iter().enumerate(){let n=i+1;let expected=if [87,89,91,92,93,94].contains(&n){1}else if [97,99,101,102,103,104].contains(&n){2}else{0};if f!=expected{return Err("unexpected menu frame");}}
  let mut states:[Vec<Batch>;6]=std::array::from_fn(|_|vec![]);
  for batches in &mut states{let n=take::<1>(bytes,&mut pos)?[0];if !(1..=4).contains(&n){return Err("invalid menu batch count");}
   for _ in 0..n{let rgba=take(bytes,&mut pos)?;let n=u16::from_le_bytes(take(bytes,&mut pos)?)as usize;if n==0||n%3!=0||n>2048{return Err("invalid menu vertices");}let mut vertices=vec![];for _ in 0..n{let x=f32::from_le_bytes(take(bytes,&mut pos)?);let y=f32::from_le_bytes(take(bytes,&mut pos)?);if !x.is_finite()||!y.is_finite()||x.abs()>512.||y.abs()>512.{return Err("invalid menu vertex");}vertices.push([x,y]);}batches.push(Batch{rgba,vertices});}
  }if pos!=bytes.len(){return Err("trailing menu bytes");}Ok(Self{instances,frames,states})
 }
 pub fn group(&self,frame:u16)->u8{self.frames[usize::from(frame.clamp(1,105)-1)]}
 pub fn layers(&self,r:&Renderer,d:&wgpu::Device,q:&wgpu::Queue,group:u8,selected:u8,pressed:bool)->Vec<Layer>{
  if !(1..=2).contains(&group){return vec![];}let g=usize::from(group-1);let mut batches:std::collections::BTreeMap<(usize,[u8;4]),Vec<[f32;2]>>=std::collections::BTreeMap::new();
  for instance in &self.instances[g]{let state=if instance.id==selected{if pressed{2}else{1}}else{0};for (order,b) in self.states[g*3+state].iter().enumerate(){batches.entry((order,b.rgba)).or_default().extend(b.vertices.iter().map(|p|[p[0]+instance.x,p[1]+instance.y]));}}
  batches.into_iter().filter_map(|((_,rgba),v)|r.mesh(d,q,rgba,&v)).collect()
 }
}
#[derive(Default)]pub struct MenuCache{key:Option<(u8,u8,bool)>,layers:Vec<Layer>}
impl MenuCache{
 pub fn update(&mut self,p:&MenuPack,r:&Renderer,d:&wgpu::Device,q:&wgpu::Queue,frame:u16,selected:u8,pressed:bool){let group=p.group(frame);let key=(group,selected,pressed);if self.key==Some(key){return;}self.layers=p.layers(r,d,q,group,selected,pressed);self.key=Some(key);}
 pub fn layers(&self)->&[Layer]{&self.layers}
}
#[cfg(test)]mod tests{use super::*;#[test]fn rejects_truncated(){for b in [b"".as_slice(),b"PKM1",b"PKM1\x05"]{assert!(MenuPack::read(b).is_err());}}}
