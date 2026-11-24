//! Shared planning for original raster layers while vector conversion proceeds.
use crate::{App,Scene};
fn art(kind:&str,frame:u16)->String{format!("art/{kind}/{frame}.png")}
#[derive(Debug,Default)]pub struct AssetPlan{pub current:Vec<String>,pub ahead:Vec<String>}
impl AssetPlan{
 pub fn for_app(app:&App,layered:bool,photo:&str)->Self{
  let mut p=Self::default();let scene=app.scene();let kind=if layered{"foreground"}else{"presskit"};
  match scene{
   Scene::Launcher=>p.current.push(art("launcher",1)),
   Scene::Gallery=>{if !photo.is_empty(){p.current.push(photo.into());}p.current.push(art("gallery_controls",1));},
   Scene::Video=>p.current.push(art("video_controls",1)),
   Scene::Intro=>{},
   _=>{if layered&&app.frame>=56{p.current.push(art("background",app.background.frame));}p.current.push(art(kind,app.frame));}
  }
  match scene{
   Scene::Launcher|Scene::Intro=>for f in 2..=8{p.ahead.push(art(kind,f));},
   Scene::Transition=>{for n in 1..=5{let f=if app.rewinding{app.frame.saturating_sub(n)}else{app.frame+n};if (2..=105).contains(&f){p.ahead.push(art(kind,f));}}},
   Scene::Welcome|Scene::Wallpapers|Scene::Videos|Scene::Screens=>{
    for f in [76,86,96]{p.ahead.push(art(kind,f));}
    if scene==Scene::Videos{p.ahead.push(art(kind,92));p.ahead.push(art("video_controls",1));}
    if scene==Scene::Screens{p.ahead.push(art(kind,102));p.ahead.push(art("gallery_controls",1));}
   },
   Scene::Video|Scene::Gallery=>{let start=if scene==Scene::Video{91}else{101};for f in start..start+4{p.ahead.push(art(kind,f));}if scene==Scene::Gallery{for i in [app.gallery.index.saturating_sub(1),app.gallery.index+1]{if let Some(s)=app.gallery.paths.get(i){if s!=photo{p.ahead.push(s.clone());}}}}}
  }
  if layered&&app.frame>=56&&!matches!(scene,Scene::Video|Scene::Gallery){for n in 1..=4{p.ahead.insert((n-1)as usize,art("background",(app.background.frame+n-1)%221+1));}}
  p.ahead.retain(|s|!p.current.contains(s));p
 }
}
#[cfg(test)]mod tests{use super::*;use crate::Hit;
 #[test]fn original_bounds_and_wrap(){let mut a=App::default();let p=AssetPlan::for_app(&a,true,"");assert_eq!(p.current,["art/launcher/1.png"]);a.activate(Hit::Accept,0.);a.media_complete();for _ in 0..74{a.tick();}a.background.frame=220;let p=AssetPlan::for_app(&a,true,"");assert_eq!(p.current,["art/background/220.png","art/foreground/75.png"]);assert!(p.ahead.contains(&"art/background/221.png".into()));assert!(p.ahead.contains(&"art/background/1.png".into()));assert!(!p.ahead.contains(&"art/background/222.png".into()));}
 #[test]fn rewind_prefetches_backward(){let mut a=App::default();a.launched=true;a.frame=94;a.rewinding=true;let p=AssetPlan::for_app(&a,false,"");assert_eq!(p.ahead[0],"art/presskit/93.png");assert!(!p.ahead.contains(&"art/presskit/95.png".into()));}
}
