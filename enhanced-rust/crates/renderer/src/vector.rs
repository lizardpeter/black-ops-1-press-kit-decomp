//! Direct GPU vector contours, in original twips (20 twips = 1 stage pixel).
//! Scope: solid simple polygons only; no gradients, masks, fonts, or curve decoding yet.
use crate::{Renderer,Layer};
const GOLD:[u8;4]=[242,180,7,255];
const DARK:[u8;4]=[17,31,41,255];
// DefineShape2 #170 @2729119 and #161 @2728279: normal play/pause skins.
const BUTTON:[[f32;2];4]=[[1320.,0.],[0.,0.],[0.,620.],[1320.,620.]];
const PLAY:[[f32;2];3]=[[577.,120.],[577.,520.],[852.,320.]];
const PAUSE_L:[[f32;2];4]=[[660.,520.],[660.,120.],[560.,120.],[560.,520.]];
const PAUSE_R:[[f32;2];4]=[[849.,520.],[849.,120.],[749.,120.],[749.,520.]];
// DefineShape #25 @638789; time field background at control origin.
const TIME:[[f32;2];4]=[[15080.,0.],[12560.,0.],[12560.,620.],[15080.,620.]];
// DefineShape #160 @2728226; eight-point concave lower track.
const TRACK:[[f32;2];8]=[[17295.,1319.],[17600.,1320.],[17540.,1220.],[59.,1220.],[0.,1320.],[304.,1320.],[340.,1260.],[17260.,1260.]];
// DefineShape3 #179 @2729933, progress_mc placed (4,48) in sprite 191.
const PROGRESS:[[f32;2];4]=[[17440.,0.],[0.,0.],[0.,200.],[17440.,200.]];
fn cross(a:[f32;2],b:[f32;2],c:[f32;2])->f32{(b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])}
/// Ear clipping for a simple closed contour without holes; returns triangle-list vertices.
pub fn triangulate(points:&[[f32;2]])->Option<Vec<[f32;2]>>{
    if points.len()<3||points.iter().flatten().any(|x|!x.is_finite()){return None;}
    let area:f64=(0..points.len()).map(|i|{let a=points[i];let b=points[(i+1)%points.len()];a[0] as f64*b[1] as f64-a[1] as f64*b[0] as f64}).sum();
    if area.abs()<1e-6{return None;}
    let sign=area.signum() as f32;let mut ids:Vec<_>=(0..points.len()).collect();let mut out=vec![];
    while ids.len()>3{
        let mut ear=None;
        for i in 0..ids.len(){let a=ids[(i+ids.len()-1)%ids.len()];let b=ids[i];let c=ids[(i+1)%ids.len()];
            if sign*cross(points[a],points[b],points[c])<=1e-6{continue;}
            if ids.iter().any(|&j|j!=a&&j!=b&&j!=c&&sign*cross(points[a],points[b],points[j])>=0.&&sign*cross(points[b],points[c],points[j])>=0.&&sign*cross(points[c],points[a],points[j])>=0.){continue;}
            ear=Some((i,a,b,c));break;
        }
        let (i,a,b,c)=ear?;out.extend([points[a],points[b],points[c]]);ids.remove(i);
    }
    out.extend(ids.iter().map(|i|points[*i]));Some(out)
}
impl Renderer{
    /// Replace the baked play/pause skin and time field with original vector geometry.
    /// Progress width is an enhanced native mapping of the original progress shape.
    pub fn gallery_vectors(&self,device:&wgpu::Device,queue:&wgpu::Queue,y:f32,text:&str)->Vec<Layer>{
        let mut layers=vec![self.layer(device,queue,&GOLD,1,1,[861.,y,93.,31.])];
        if let Some(t)=self.counter(device,queue,text,y,true){layers.push(t);}layers
    }
    pub fn video_vectors(&self,device:&wgpu::Device,queue:&wgpu::Queue,y:f32,paused:bool,progress:f32)->Vec<Layer>{
        let mut layers=vec![];
        let mut add=|p:&[[f32;2]],x:f32,dy:f32,sx:f32,color:[u8;4]|{
            let pts:Vec<_>=p.iter().map(|v|[x+v[0]*sx/20.,y+dy+v[1]/20.]).collect();
            if let Some(l)=self.polygon(device,queue,color,&pts){layers.push(l);}
        };
        add(&BUTTON,215.,0.,1.,GOLD);
        if paused{add(&PLAY,215.,0.,1.,DARK);}else{add(&PAUSE_L,215.,0.,1.,DARK);add(&PAUSE_R,215.,0.,1.,DARK);}
        add(&TIME,200.,0.,1.,GOLD);
        add(&TRACK,200.,0.,1.,[240,240,240,255]);
        if progress>0.{add(&PROGRESS,204.,48.,progress.clamp(0.,1.),GOLD);}
        layers
    }
}
#[cfg(test)]mod tests{use super::*;#[test]fn original_contours(){for p in [&BUTTON[..],&PLAY,&PAUSE_L,&PAUSE_R,&TIME,&TRACK,&PROGRESS]{let t=triangulate(p).unwrap();assert_eq!(t.len(),(p.len()-2)*3);let area:f32=t.chunks_exact(3).map(|a|cross(a[0],a[1],a[2]).abs()).sum();let expected:f32=(0..p.len()).map(|i|{let a=p[i];let b=p[(i+1)%p.len()];a[0]*b[1]-a[1]*b[0]}).sum::<f32>().abs();assert!((area-expected).abs()<10.);}}}
