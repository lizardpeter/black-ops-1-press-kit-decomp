//! Source-derived reusable foreground sprite textures and root placements.
use crate::{Layer,Renderer};
use crate::menu::{MenuPack,MenuCache};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Placement{pub image:usize,pub x:i32,pub y:i32,pub width:u32,pub height:u32,pub alpha:u16}
impl Placement{pub fn rect(self)->[f32;4]{[self.x as f32/20.,self.y as f32/20.,self.width as f32/20.,self.height as f32/20.]}}
struct MeshBatch{rgba:[u8;4],vertices:Vec<[f32;2]>}
pub struct ForegroundTimeline{vector_menu:bool,explicit_menu_depth:bool,dimensions:Vec<(u32,u32)>,codecs:Vec<u8>,meshes:Vec<Vec<MeshBatch>>,menu_after:Vec<u8>,frames:Vec<Vec<Placement>>}
impl ForegroundTimeline{
 pub fn read(bytes:&[u8])->Result<Self,&'static str>{
  let v4=bytes.get(..4)==Some(b"PFG4");let v3=v4||bytes.get(..4)==Some(b"PFG3");let vector_menu=v3||bytes.get(..4)==Some(b"PFG2");if (!vector_menu&&bytes.get(..4)!=Some(b"PFG1"))||bytes.len()>if v4{8*1024*1024}else{65536}{return Err("invalid foreground timeline");}let mut p=4;
  fn take<const N:usize>(b:&[u8],p:&mut usize)->Result<[u8;N],&'static str>{let end=p.checked_add(N).ok_or("timeline overflow")?;let data=b.get(*p..end).ok_or("truncated foreground")?;*p=end;Ok(data.try_into().unwrap())}
  let count=u16::from_le_bytes(take(bytes,&mut p)?) as usize;let frames=u16::from_le_bytes(take(bytes,&mut p)?);if !(1..=256).contains(&count)||frames!=105{return Err("invalid foreground counts");}
  let mut dimensions=Vec::with_capacity(count);let mut memory=0u64;let mut codecs=Vec::with_capacity(count);for _ in 0..count{let w=u32::from(u16::from_le_bytes(take(bytes,&mut p)?));let h=u32::from(u16::from_le_bytes(take(bytes,&mut p)?));if w==0||h==0||w>4096||h>4096{return Err("invalid sprite dimensions");}memory+=u64::from(w)*u64::from(h)*4;dimensions.push((w,h));let codec=if v3{take::<1>(bytes,&mut p)?[0]}else{0};if codec>if v4{2}else{1}{return Err("invalid foreground codec");}if codec==2&&(w,h)!=(1,1){return Err("invalid mesh placeholder");}codecs.push(codec);}if memory>96*1024*1024{return Err("foreground texture budget exceeded");}
  let mut timeline=Vec::with_capacity(105);let mut menu_after=Vec::with_capacity(105);for _ in 0..frames{let n=take::<1>(bytes,&mut p)?[0];let after=if v3{take::<1>(bytes,&mut p)?[0]}else{4};if v3&&after!=255&&(after==0||after>n){return Err("invalid menu insertion");}menu_after.push(after);if n>64{return Err("invalid placement count");}let mut placements=vec![];
   for _ in 0..n{let a=Placement{image:u16::from_le_bytes(take(bytes,&mut p)?) as usize,x:i32::from_le_bytes(take(bytes,&mut p)?),y:i32::from_le_bytes(take(bytes,&mut p)?),width:u32::from_le_bytes(take(bytes,&mut p)?),height:u32::from_le_bytes(take(bytes,&mut p)?),alpha:u16::from_le_bytes(take(bytes,&mut p)?)};
    if a.image>=count||a.alpha>256||a.width==0||a.height==0||a.width>100000||a.height>100000||a.x.unsigned_abs()>100000||a.y.unsigned_abs()>100000||placements.iter().any(|b:&Placement|b.image==a.image){return Err("invalid or duplicate sprite placement");}placements.push(a);}
   timeline.push(placements);
  }
  let mut meshes:Vec<Vec<MeshBatch>>=(0..count).map(|_|vec![]).collect();let mut total_vertices=0usize;
  for (i,&codec) in codecs.iter().enumerate(){if codec!=2{continue;}let n=u16::from_le_bytes(take(bytes,&mut p)?) as usize;if !(1..=128).contains(&n){return Err("invalid mesh batch count");}for _ in 0..n{let rgba=take(bytes,&mut p)?;let n=u32::from_le_bytes(take(bytes,&mut p)?) as usize;if n==0||n%3!=0||n>500_000{return Err("invalid foreground mesh size");}total_vertices+=n;if total_vertices>900_000{return Err("foreground mesh budget exceeded");}let mut vertices=Vec::with_capacity(n);for _ in 0..n{let x=f32::from_le_bytes(take(bytes,&mut p)?);let y=f32::from_le_bytes(take(bytes,&mut p)?);if !x.is_finite()||!y.is_finite()||x.abs()>4096.||y.abs()>4096.{return Err("invalid foreground mesh vertex");}vertices.push([x,y]);}meshes[i].push(MeshBatch{rgba,vertices});}}
  if p!=bytes.len(){return Err("trailing foreground data");}Ok(Self{vector_menu,explicit_menu_depth:v3,dimensions,codecs,meshes,menu_after,frames:timeline})
 }
 pub fn frame(&self,n:u16)->&[Placement]{&self.frames[usize::from(n.clamp(1,105)-1)]}
 pub fn filename(&self,i:usize)->String{format!("{i}.{}",match self.codecs.get(i){Some(1)=>"jpg",Some(2)=>"mesh",_=>"png"})}
 pub fn validate_menu(&self,menu:&MenuPack)->Result<(),&'static str>{
  if !self.vector_menu{return Err("menu pack requires PFG2 or PFG3");}
  if self.explicit_menu_depth{for f in 1..=105u16{let active=self.menu_after[usize::from(f-1)]!=255;if active!=(menu.group(f)!=0){return Err("menu activation does not match foreground depth map");}}}
  Ok(())
 }
 pub fn count(&self)->usize{self.dimensions.len()}
 pub fn is_mesh(&self,i:usize)->bool{self.codecs.get(i)==Some(&2)}
}
pub struct ForegroundArt{timeline:ForegroundTimeline,textures:Vec<Option<Layer>>,last:Option<u16>,menu:Option<MenuPack>,menu_cache:MenuCache,mesh_frame:Option<u16>,mesh_layers:std::collections::HashMap<usize,Vec<Layer>>}
impl ForegroundArt{
 pub fn new(timeline:ForegroundTimeline)->Self{let textures=(0..timeline.count()).map(|_|None).collect();Self{timeline,textures,last:None,menu:None,menu_cache:MenuCache::default(),mesh_frame:None,mesh_layers:std::collections::HashMap::new()}}
 pub fn filename(&self,i:usize)->String{self.timeline.filename(i)}
 pub fn needs_menu(&self)->bool{self.timeline.vector_menu&&self.menu.is_none()}
 pub fn install_menu(&mut self,bytes:&[u8])->Result<(),&'static str>{let menu=MenuPack::read(bytes)?;self.timeline.validate_menu(&menu)?;self.menu=Some(menu);self.menu_cache=MenuCache::default();Ok(())}
 pub fn update_menu(&mut self,r:&Renderer,d:&wgpu::Device,q:&wgpu::Queue,selected:u8,pressed:bool){if let (Some(menu),Some(frame))=(&self.menu,self.last){self.menu_cache.update(menu,r,d,q,frame,selected,pressed);}
  if self.mesh_frame!=self.last{self.mesh_layers.clear();if let Some(frame)=self.last{for p in self.timeline.frame(frame){if !self.timeline.is_mesh(p.image){continue;}let [x,y,_,_]=p.rect();let layers=self.timeline.meshes[p.image].iter().filter_map(|b|{let points:Vec<_>=b.vertices.iter().map(|v|[v[0]+x,v[1]+y]).collect();r.mesh_with_opacity(d,b.rgba,&points,p.alpha as f32/256.)}).collect();self.mesh_layers.insert(p.image,layers);}}self.mesh_frame=self.last;}}
 pub fn install(&mut self,i:usize,layer:Layer)->Result<(),&'static str>{if self.timeline.is_mesh(i){return Err("mesh slot cannot receive a texture");}if self.timeline.dimensions.get(i)!=Some(&(layer.texture.width(),layer.texture.height())){return Err("unexpected sprite dimensions");}if self.textures[i].is_some()&&self.last.is_some_and(|f|self.timeline.frame(f).iter().any(|p|p.image==i)){self.last=None;}self.textures[i]=Some(layer);Ok(())}
 pub fn missing(&self)->Vec<usize>{self.textures.iter().enumerate().filter_map(|(i,v)|(v.is_none()&&!self.timeline.is_mesh(i)).then_some(i)).collect()}
 pub fn missing_frame(&self,frame:u16)->Vec<usize>{self.timeline.frame(frame).iter().filter_map(|p|(self.textures[p.image].is_none()&&!self.timeline.is_mesh(p.image)).then_some(p.image)).collect()}
 pub fn ready_frame(&self,frame:u16)->bool{!self.needs_menu()&&self.missing_frame(frame).is_empty()}
 pub fn update(&mut self,r:&Renderer,q:&wgpu::Queue,frame:u16){if self.last==Some(frame)||!self.ready_frame(frame){return;}for p in self.timeline.frame(frame){if self.timeline.is_mesh(p.image){continue;}r.set_opacity(q,self.textures[p.image].as_ref().unwrap(),p.rect(),p.alpha as f32/256.);}self.last=Some(frame);}
 pub fn layers(&self)->Vec<&Layer>{let Some(f)=self.last else{return vec![];};let mut layers=vec![];for (i,p) in self.timeline.frame(f).iter().enumerate(){if let Some(l)=self.textures[p.image].as_ref(){layers.push(l);}if let Some(mesh)=self.mesh_layers.get(&p.image){layers.extend(mesh.iter());}if i+1==usize::from(self.timeline.menu_after[usize::from(f-1)])&&self.menu.as_ref().is_some_and(|m|m.group(f)>0){layers.extend(self.menu_cache.labels());layers.extend(self.menu_cache.layers());}}layers}
}
#[cfg(test)]mod tests{use super::*;#[test]fn rejects_bad_inputs(){let mut b=b"PFG1\x01\0\x69\0\x01\0\x01\0".to_vec();b.extend([0;105]);let t=ForegroundTimeline::read(&b).unwrap();assert_eq!(t.count(),1);assert!(t.frame(105).is_empty());b.push(0);assert!(ForegroundTimeline::read(&b).is_err());b.pop();b[8]=0;assert!(ForegroundTimeline::read(&b).is_err());}}
