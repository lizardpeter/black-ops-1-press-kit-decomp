//! Source launcher with GPU clipped bitmap quads and native glyph triangles.
use crate::{Layer,Renderer,control_pack::ControlPack};
#[derive(Clone,Copy)]struct State{rect:[f32;4],uv:[f32;4]}
struct Bitmap{selector:u8,states:[State;3],layer:Layer}
pub struct LauncherArt{bitmaps:Vec<Bitmap>,glyphs:Vec<Layer>,last:Option<(u8,bool)>}
impl LauncherArt{
 pub fn read(b:&[u8],r:&Renderer,d:&wgpu::Device,q:&wgpu::Queue)->Result<Self,&'static str>{
  if b.get(..4)!=Some(b"PKL2")||b.len()>4*1024*1024{return Err("invalid native launcher");}
  let mut p=4usize;
  fn take<'a>(b:&'a[u8],p:&mut usize,n:usize)->Result<&'a[u8],&'static str>{let end=p.checked_add(n).ok_or("launcher overflow")?;let s=b.get(*p..end).ok_or("truncated launcher")?;*p=end;Ok(s)}
  let n=u32::from_le_bytes(take(b,&mut p,4)?.try_into().unwrap())as usize;
  let glyphs=ControlPack::read_launcher(take(b,&mut p,n)?)?.launcher_layers(r,d,q);
  if take(b,&mut p,1)?[0]!=4{return Err("invalid launcher bitmap count");}
  let mut bitmaps=vec![];
  for selector in [0,1,2,0]{
   if take(b,&mut p,1)?[0]!=selector{return Err("invalid launcher depth order");}
   let w=u16::from_le_bytes(take(b,&mut p,2)?.try_into().unwrap())as u32;
   let h=u16::from_le_bytes(take(b,&mut p,2)?.try_into().unwrap())as u32;
   if w==0||h==0||w>1024||h>1024{return Err("invalid launcher bitmap dimensions");}
   let mut states=[State{rect:[0.;4],uv:[0.;4]};3];
   for s in &mut states{
    for x in s.rect.iter_mut().chain(s.uv.iter_mut()){*x=f32::from_le_bytes(take(b,&mut p,4)?.try_into().unwrap());if !x.is_finite(){return Err("invalid launcher coordinate");}}
    let [x,y,w,h]=s.rect;let [u,v,ur,vr]=s.uv;
    if x<0.||y<0.||w<=0.||h<=0.||x+w>500.001||y+h>300.001||u<0.||v<0.||ur>1.001||vr>1.001||u>=ur||v>=vr{return Err("invalid launcher clipping state");}
    s.rect[0]+=390.;s.rect[1]+=210.;
   }
   let rgba=take(b,&mut p,(w*h*4)as usize)?;
   let layer=r.layer(d,q,rgba,w,h,states[0].rect);
   bitmaps.push(Bitmap{selector,states,layer});
  }
  if p!=b.len(){return Err("trailing launcher data");}
  let mut a=Self{bitmaps,glyphs,last:None};a.update(r,q,0,false);Ok(a)
 }
 pub fn update(&mut self,r:&Renderer,q:&wgpu::Queue,selected:u8,pressed:bool){
  if self.last==Some((selected,pressed)){return;}self.last=Some((selected,pressed));
  for b in &self.bitmaps{let s=b.states[if b.selector!=0&&b.selector==selected{if pressed{2}else{1}}else{0}];r.set_opacity_uv(q,&b.layer,s.rect,1.,s.uv);}
 }
 pub fn layers(&self)->impl Iterator<Item=&Layer>{self.bitmaps.iter().map(|b|&b.layer).chain(self.glyphs.iter())}
}
