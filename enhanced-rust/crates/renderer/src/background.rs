//! Four original JPEG textures and sprite-75 opacity frames, no baked animation images.
use crate::{Layer,Renderer};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Exposure{pub image:usize,pub alpha:u16}
pub struct BackgroundTimeline{frames:Vec<Vec<Exposure>>}
impl BackgroundTimeline{
 pub fn read(bytes:&[u8])->Result<Self,&'static str>{
  if bytes.get(..6)!=Some(b"PBG1\xdd\0"){return Err("invalid background timeline");}
  let mut p=6;let mut frames=Vec::with_capacity(221);
  for _ in 0..221{let n=*bytes.get(p).ok_or("truncated background")?;p+=1;if !(1..=2).contains(&n){return Err("invalid layer count");}let mut f=vec![];
   for _ in 0..n{let b=bytes.get(p..p+3).ok_or("truncated exposure")?;p+=3;let e=Exposure{image:b[0] as usize,alpha:u16::from_le_bytes([b[1],b[2]])};if e.image>=4||e.alpha>256||f.iter().any(|x:&Exposure|x.image==e.image){return Err("invalid exposure");}f.push(e);}frames.push(f);
  }if p!=bytes.len(){return Err("trailing background data");}Ok(Self{frames})
 }
 pub fn frame(&self,n:u16)->&[Exposure]{&self.frames[usize::from(n.clamp(1,221)-1)]}
}
pub struct BackgroundArt{timeline:BackgroundTimeline,textures:[Option<Layer>;4],last:u16,pair:Option<Layer>,pair_ids:Option<(usize,usize)>}
impl BackgroundArt{
 pub fn new(timeline:BackgroundTimeline)->Self{Self{timeline,textures:std::array::from_fn(|_|None),last:0,pair:None,pair_ids:None}}
 pub fn install(&mut self,i:usize,layer:Layer)->Result<(),&'static str>{if i>=4||layer.texture.width()!=1280||layer.texture.height()!=720{return Err("invalid original background texture");}self.textures[i]=Some(layer);self.last=0;self.pair=None;self.pair_ids=None;Ok(())}
 pub fn ready(&self)->bool{self.textures.iter().all(Option::is_some)}
 pub fn missing(&self)->Vec<usize>{(0..4).filter(|&i|self.textures[i].is_none()).collect()}
 pub fn update(&mut self,r:&Renderer,d:&wgpu::Device,q:&wgpu::Queue,frame:u16){
  if self.last==frame{return;}let f=self.timeline.frame(frame);let a=f[0];let b=f.get(1).copied().unwrap_or(Exposure{image:a.image,alpha:0});
  let Some(first)=&self.textures[a.image] else{self.pair=None;self.pair_ids=None;return;};let Some(second)=&self.textures[b.image] else{self.pair=None;self.pair_ids=None;return;};
  if self.pair_ids!=Some((a.image,b.image)){self.pair=Some(r.background_pair(d,first,second));self.pair_ids=Some((a.image,b.image));}
  if let Some(l)=&self.pair{r.set_background_opacity(q,l,[a.alpha as f32/256.,b.alpha as f32/256.]);}self.last=frame;
 }
 pub fn layers(&self,_frame:u16)->Vec<&Layer>{self.pair.iter().collect()}

}
#[cfg(test)]mod tests{use super::*;#[test]fn checks_bounds(){let mut b=b"PBG1\xdd\0".to_vec();for _ in 0..221{b.extend([1,0,0,1]);}let t=BackgroundTimeline::read(&b).unwrap();assert_eq!(t.frame(221),[Exposure{image:0,alpha:256}]);b[7]=4;assert!(BackgroundTimeline::read(&b).is_err());}}
