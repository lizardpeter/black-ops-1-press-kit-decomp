//! Original DefineFont3 #30 (Molot), numeric counter subset; units: 20480/em.
use crate::{Layer,Renderer};
fn glyph(c:char)->Option<(f64,&'static str)>{Some(match c{
 ' ' => (10040.,"1000"),
 '/' => (10760.,"100dea7fa58e704f78531a727652db967a7b3d912c73b166dcac330400"),
 '0' => (11760.,"100de6542b10704e8b50f16fe3a12c3c6408c1017a3c8000182784cad54b87c97c94bd7ad5f8a81ba5e9a43e4bc19aaa9a401b81e1a1bd550455f38d0d2b80dd8200"),
 '1' => (6120.,"100dca500001c13c2e80f59701dd195c90ec45b7ab7f0c1000"),
 '2' => (10940.,"100de91a329f704f37591361cac0bc1608ed7087a5878f1b552af6a9ac550ce0f28ac6a5dcb44bc2cbeeba611c9d3219e8822aa69006e06e686ed65e52051a97104754f1a475420dc60800"),
 '3' => (11340.,"100dea5032d1704f14d5aaf9269a97006ef4bed9db1f0f0584ba613bb26b1db754ee94cbbe72d12ec80875d3a8e4e990cf44e85dad5cef92a47e54dd40dc078d998200"),
 '4' => (12040.,"100de2d7a408704ec7c174b7579e201568e55ddd0dc1db5e2f04603d5167785d01db560f09d1bc6edaf24ec47ad82000"),
 '5' => (11560.,"100de0c82408704f44ec1d2de9d7511c73c2cdce9640ec79954d200dc0dcd35e2ac855f24d352e00dde97d3bb63e1e0b1874c407853a1eb768304000"),
 '6' => (11560.,"100dcca86431c13a6113a2d43a5ef3a12c30405e8f22e3a609535203203234878ab4957ce3434ae037784cad54b87c97c94bd7ad5f8a81ba5e9a43e4bc1ffee96f4eba88e39e16a074b2a763ca304000"),
 '7' => (9420.,"100de4512c05704e185bc2f84ed8f87a226aed7307be4f8a643c2fc0e9b82f62f5ae44608000"),
 '8' => (11760.,"100dca5054b9c13a5ef3a12c3a6113a2d430405e5282b10609d2ca9d0961d3359d16a18202d514000304aa573e4be725ebb63b948149d484b1a5284b1a2f8fedc33353423fb5eec694deec7dfb27076cc9aa07897a690f92f066aaa69006e06e686ed66d53fb1905f83854dee3994200570b354af8014848e652820e601462ddae24abe49a1a5c01bbc2656c1000"),
 '9' => (11560.,"100de5002b0b704e979ee8460e9862e8ba0c101712fd5d360954b87ce7c94b378d4a550374c23487c97832b5535203703233e7aa9feabe71a6a5701bbc21a6ed8f87827c5d336dd911ec1000"),
 ':' => (5680.,"100da32a8f8704ec7ee76b843b2049db1f0c1017832a902182763f73b5c21d9024ed8f860800"),
 _=>return None})}
type Point=[f64;2];
struct Bits{bytes:Vec<u8>,at:usize}
impl Bits{
 fn read(&mut self,n:usize)->Option<u32>{if n>32||self.at+n>self.bytes.len()*8{return None;}let mut v=0;for _ in 0..n{v=(v<<1)|u32::from((self.bytes[self.at/8]>>(7-self.at%8))&1);self.at+=1;}Some(v)}
 fn signed(&mut self,n:usize)->Option<f64>{let v=self.read(n)? as i64;Some(if n>0&&v&(1<<(n-1))!=0{v-(1<<n)}else{v} as f64)}
}
fn midpoint(a:Point,b:Point)->Point{[(a[0]+b[0])*0.5,(a[1]+b[1])*0.5]}
fn curve(a:Point,b:Point,c:Point,out:&mut Vec<Point>,depth:u8){let ab=midpoint(a,b);let bc=midpoint(b,c);let m=midpoint(ab,bc);let chord=midpoint(a,c);if depth>=12||((m[0]-chord[0]).abs()+(m[1]-chord[1]).abs())<=8.{out.push(c);return;}curve(a,ab,m,out,depth+1);curve(m,bc,c,out,depth+1);}
fn contours(hex:&str)->Option<Vec<Vec<Point>>>{
 if hex.len()%2!=0{return None;}let bytes=(0..hex.len()).step_by(2).map(|i|u8::from_str_radix(&hex[i..i+2],16).ok()).collect::<Option<Vec<_>>>()?;
 let mut r=Bits{bytes,at:0};let nf=r.read(4)? as usize;let nl=r.read(4)? as usize;let mut at=[0.,0.];let mut paths=vec![];let mut p=vec![];
 loop{if r.read(1)?==1{let straight=r.read(1)?==1;let n=r.read(4)? as usize+2;if p.is_empty(){p.push(at);}if straight{if r.read(1)?==1{at[0]+=r.signed(n)?;at[1]+=r.signed(n)?;}else if r.read(1)?==1{at[1]+=r.signed(n)?;}else{at[0]+=r.signed(n)?;}p.push(at);}else{let b=[at[0]+r.signed(n)?,at[1]+r.signed(n)?];let c=[b[0]+r.signed(n)?,b[1]+r.signed(n)?];curve(at,b,c,&mut p,0);at=c;}}
 else{let flags=r.read(5)?;if flags==0{break;}if flags&16!=0{return None;}if flags&1!=0{if !p.is_empty(){paths.push(std::mem::take(&mut p));}let n=r.read(5)? as usize;at=[r.signed(n)?,r.signed(n)?];}if flags&2!=0{r.read(nf)?;}if flags&4!=0{r.read(nf)?;}if flags&8!=0{r.read(nl)?;}}}
 if !p.is_empty(){paths.push(p);}if paths.iter().any(|p|p.len()<3||p.first()!=p.last()){return None;}Some(paths)
}
/// Trapezoid decomposition for non-crossing closed contours, including holes.
fn tessellate(paths:&[Vec<Point>])->Option<Vec<Point>>{
 let mut ys:Vec<_>=paths.iter().flatten().map(|p|p[1]).collect();ys.sort_by(f64::total_cmp);ys.dedup();let mut edges=vec![];for p in paths{for e in p.windows(2){if e[0][1]!=e[1][1]{edges.push((e[0],e[1]));}}}
 let x_at=|e:(Point,Point),y:f64|e.0[0]+(e.1[0]-e.0[0])*(y-e.0[1])/(e.1[1]-e.0[1]);let mut out=vec![];
 for band in ys.windows(2){let (y0,y1)=(band[0],band[1]);let mid=(y0+y1)*0.5;let mut active:Vec<_>=edges.iter().copied().filter(|(a,b)|mid>a[1].min(b[1])&&mid<a[1].max(b[1])).collect();active.sort_by(|a,b|x_at(*a,mid).total_cmp(&x_at(*b,mid)));if active.len()%2!=0{return None;}for e in active.chunks_exact(2){let a=[x_at(e[0],y0),y0];let b=[x_at(e[1],y0),y0];let c=[x_at(e[0],y1),y1];let d=[x_at(e[1],y1),y1];out.extend([a,c,b,b,c,d]);}}Some(out)
}
pub fn counter_mesh(text:&str,right:f64,top:f64,size:f64)->Option<Vec<[f32;2]>>{
 if !size.is_finite()||size<=0.||!right.is_finite()||!top.is_finite(){return None;}let scale=size/20480.;let width=text.chars().map(|c|glyph(c).map(|g|g.0)).collect::<Option<Vec<_>>>()?.iter().sum::<f64>()*scale;let mut x=right-width;let baseline=top+19360.*scale;let mut vertices=vec![];for c in text.chars(){let (advance,hex)=glyph(c)?;let triangles=tessellate(&contours(hex)?)?;vertices.extend(triangles.iter().map(|p|[(x+p[0]*scale) as f32,(baseline+p[1]*scale) as f32]));x+=advance*scale;}Some(vertices)
}
impl Renderer{
 pub fn counter(&self,device:&wgpu::Device,queue:&wgpu::Queue,text:&str,y:f32,gallery:bool)->Option<Layer>{let right=941.;let top=f64::from(y)+if gallery{5.}else{6.};let mesh=counter_mesh(text,right,top,16.)?;self.mesh(device,queue,if gallery{[17,31,41,255]}else{[0,0,0,255]},&mesh)}
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn all_original_counter_glyphs(){for c in "0123456789:/ ".chars(){let (_,hex)=glyph(c).unwrap();let p=contours(hex).unwrap_or_else(||panic!("contour {c}"));let t=tessellate(&p).unwrap();assert_eq!(t.len()%3,0);if c!=' '{assert!(!t.is_empty(),"{c}");}}}
 #[test]fn holes_stay_open(){let p=vec![vec![[0.,0.],[10.,0.],[10.,10.],[0.,10.],[0.,0.]],vec![[3.,3.],[7.,3.],[7.,7.],[3.,7.],[3.,3.]]];let t=tessellate(&p).unwrap();let area:f64=t.chunks_exact(3).map(|t|((t[1][0]-t[0][0])*(t[2][1]-t[0][1])-(t[1][1]-t[0][1])*(t[2][0]-t[0][0])).abs()/2.).sum();assert!((area-84.).abs()<1e-8);}
 #[test]fn scalable_counter(){let a=counter_mesh("01:30/03:00",950.,620.,16.).unwrap();let b=counter_mesh("01:30/03:00",1900.,1240.,32.).unwrap();assert!(!a.is_empty());assert_eq!(a.len(),b.len());for (a,b) in a.iter().zip(b){assert!((a[0]*2.-b[0]).abs()<0.001);assert!((a[1]*2.-b[1]).abs()<0.001);}}
}
