//! Source-derived reusable foreground sprite textures and root placements.
use crate::{Layer,Renderer};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Placement{pub image:usize,pub x:i32,pub y:i32,pub width:u32,pub height:u32,pub alpha:u16}
impl Placement{pub fn rect(self)->[f32;4]{[self.x as f32/20.,self.y as f32/20.,self.width as f32/20.,self.height as f32/20.]}}
pub struct ForegroundTimeline{dimensions:Vec<(u32,u32)>,frames:Vec<Vec<Placement>>}
impl ForegroundTimeline{
 pub fn read(bytes:&[u8])->Result<Self,&'static str>{
  if bytes.get(..4)!=Some(b"PFG1")||bytes.len()>65536{return Err("invalid foreground timeline");}let mut p=4;
  fn take<const N:usize>(b:&[u8],p:&mut usize)->Result<[u8;N],&'static str>{let end=p.checked_add(N).ok_or("timeline overflow")?;let data=b.get(*p..end).ok_or("truncated foreground")?;*p=end;Ok(data.try_into().unwrap())}
  let count=u16::from_le_bytes(take(bytes,&mut p)?) as usize;let frames=u16::from_le_bytes(take(bytes,&mut p)?);if !(1..=256).contains(&count)||frames!=105{return Err("invalid foreground counts");}
  let mut dimensions=Vec::with_capacity(count);let mut memory=0u64;for _ in 0..count{let w=u32::from(u16::from_le_bytes(take(bytes,&mut p)?));let h=u32::from(u16::from_le_bytes(take(bytes,&mut p)?));if w==0||h==0||w>4096||h>4096{return Err("invalid sprite dimensions");}memory+=u64::from(w)*u64::from(h)*4;dimensions.push((w,h));}if memory>96*1024*1024{return Err("foreground texture budget exceeded");}
  let mut timeline=Vec::with_capacity(105);for _ in 0..frames{let n=take::<1>(bytes,&mut p)?[0];if n>32{return Err("invalid placement count");}let mut placements=vec![];
   for _ in 0..n{let a=Placement{image:u16::from_le_bytes(take(bytes,&mut p)?) as usize,x:i32::from_le_bytes(take(bytes,&mut p)?),y:i32::from_le_bytes(take(bytes,&mut p)?),width:u32::from_le_bytes(take(bytes,&mut p)?),height:u32::from_le_bytes(take(bytes,&mut p)?),alpha:u16::from_le_bytes(take(bytes,&mut p)?)};
    if a.image>=count||a.alpha>256||a.width==0||a.height==0||a.width>100000||a.height>100000||a.x.unsigned_abs()>100000||a.y.unsigned_abs()>100000||placements.iter().any(|b:&Placement|b.image==a.image){return Err("invalid or duplicate sprite placement");}placements.push(a);}
   timeline.push(placements);
  }if p!=bytes.len(){return Err("trailing foreground data");}Ok(Self{dimensions,frames:timeline})
 }
 pub fn frame(&self,n:u16)->&[Placement]{&self.frames[usize::from(n.clamp(1,105)-1)]}
 pub fn count(&self)->usize{self.dimensions.len()}
}
pub struct ForegroundArt{timeline:ForegroundTimeline,textures:Vec<Option<Layer>>,last:Option<u16>}
impl ForegroundArt{
 pub fn new(timeline:ForegroundTimeline)->Self{let textures=(0..timeline.count()).map(|_|None).collect();Self{timeline,textures,last:None}}
 pub fn install(&mut self,i:usize,layer:Layer)->Result<(),&'static str>{if self.timeline.dimensions.get(i)!=Some(&(layer.texture.width(),layer.texture.height())){return Err("unexpected sprite dimensions");}if self.textures[i].is_some()&&self.last.is_some_and(|f|self.timeline.frame(f).iter().any(|p|p.image==i)){self.last=None;}self.textures[i]=Some(layer);Ok(())}
 pub fn missing(&self)->Vec<usize>{self.textures.iter().enumerate().filter_map(|(i,v)|v.is_none().then_some(i)).collect()}
 pub fn missing_frame(&self,frame:u16)->Vec<usize>{self.timeline.frame(frame).iter().filter_map(|p|self.textures[p.image].is_none().then_some(p.image)).collect()}
 pub fn ready_frame(&self,frame:u16)->bool{self.missing_frame(frame).is_empty()}
 pub fn update(&mut self,r:&Renderer,q:&wgpu::Queue,frame:u16){if self.last==Some(frame)||!self.ready_frame(frame){return;}for p in self.timeline.frame(frame){r.set_opacity(q,self.textures[p.image].as_ref().unwrap(),p.rect(),p.alpha as f32/256.);}self.last=Some(frame);}
 pub fn layers(&self)->Vec<&Layer>{self.last.map(|f|self.timeline.frame(f).iter().filter_map(|p|self.textures[p.image].as_ref()).collect()).unwrap_or_default()}
}
#[cfg(test)]mod tests{use super::*;#[test]fn rejects_bad_inputs(){let mut b=b"PFG1\x01\0\x69\0\x01\0\x01\0".to_vec();b.extend([0;105]);let t=ForegroundTimeline::read(&b).unwrap();assert_eq!(t.count(),1);assert!(t.frame(105).is_empty());b.push(0);assert!(ForegroundTimeline::read(&b).is_err());b.pop();b[8]=0;assert!(ForegroundTimeline::read(&b).is_err());}}
