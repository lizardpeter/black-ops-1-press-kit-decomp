use crate::{Gallery,Hit,INTRO,WEBSITE,SCREEN_XML,WALLPAPERS,trailer,hit_launcher,hit_menu,hit_media};
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Scene{Launcher,Intro,Transition,Welcome,Wallpapers,Videos,Video,Screens,Gallery}
#[derive(Debug,Clone,PartialEq)]
pub enum Effect {Fullscreen,Quit,Play(&'static str),StopVideo,ToggleVideo,Seek(f64),Music(bool),Sound(u16),Open(&'static str),ReadXml(&'static str),Image(String)}
#[derive(Debug)]
pub struct Background {pub frame:u16, pub pause_ms:f64, credit:f64}
impl Default for Background {fn default()->Self{Self{frame:1,pause_ms:0.,credit:0.}}}
impl Background {
    pub fn advance(&mut self,mut ms:f64) {
        if !ms.is_finite()||ms<=0.{return;}
        while ms>0.{
            if self.pause_ms>0. {let used=ms.min(self.pause_ms);self.pause_ms-=used;ms-=used;continue;}
            let need=1000./24.-self.credit;
            if ms<need{self.credit+=ms;break;}
            ms-=need;self.credit=0.;self.frame=if self.frame==221{1}else{self.frame+1};
            if matches!(self.frame,2|57|112|167){self.pause_ms=2000.;}
        }
    }
}
#[derive(Debug)]
pub struct Controls {pub y:f64,pub target:f64,pub visible:bool,counter:u8}
impl Default for Controls {fn default()->Self{Self{y:1356.,target:770.,visible:false,counter:0}}}
impl Controls {
    pub fn activity(&mut self){self.visible=true;self.counter=0;}
    pub fn tick(&mut self){
        if self.visible{self.counter+=1;if self.counter==50{self.visible=false;self.counter=0;}self.target=620.;}
        else{self.target=770.;}
        self.y-=((self.y-self.target)/5.).ceil();
    }
}
#[derive(Debug)]
pub struct App {
    pub frame:u16,pub launched:bool,pub playing:bool,pub rewinding:bool,pub gallery:Gallery,pub background:Background,pub controls:Controls,
    video:u8,credit:f64,effects:Vec<Effect>
}
impl Default for App{fn default()->Self{Self{frame:1,launched:false,playing:false,rewinding:false,gallery:Gallery::default(),background:Background::default(),controls:Controls::default(),video:0,credit:0.,effects:vec![]}}}
impl App{
    pub fn scene(&self)->Scene{
        if !self.launched{return Scene::Launcher;}
        match self.frame{1=>Scene::Intro,75=>Scene::Welcome,81|85=>Scene::Wallpapers,91=>Scene::Videos,95=>Scene::Video,101=>Scene::Screens,105=>Scene::Gallery,_=>Scene::Transition}
    }
    /// Enhanced presentation: target the settled menu frame, not the original
    /// alternating blank/visible entry frames. The faithful activate() path
    /// remains available for original timeline playback.
    pub fn steady_menu_frame(&self,hit:Hit)->Option<u16>{
        if !matches!(self.scene(),Scene::Welcome|Scene::Wallpapers|Scene::Videos|Scene::Screens){return None;}
        match hit{Hit::Wallpapers=>Some(81),Hit::Videos=>Some(91),Hit::Screens=>Some(101),_=>None}
    }
    pub fn activate_steady_menu(&mut self,hit:Hit)->bool{
        let Some(target)=self.steady_menu_frame(hit) else{return false;};
        if self.frame!=target{
            self.frame=target;self.playing=false;self.rewinding=false;
            self.effects.push(Effect::Sound(2));
            self.enter();
        }
        true
    }
    pub fn effects(&mut self)->Vec<Effect>{std::mem::take(&mut self.effects)}
    pub fn hit(&self,x:f64,y:f64)->Hit{
        match self.scene(){Scene::Launcher=>hit_launcher(x,y),Scene::Gallery|Scene::Video=>hit_media(self.scene(),x,y,self.controls.y),s=>hit_menu(s,x,y)}
    }
    pub fn media_complete(&mut self){
        match self.scene(){Scene::Intro=>self.playing=true,Scene::Video=>self.close_media(),_=>{}}
    }
    fn close_media(&mut self){
        if self.rewinding{return;}
        if matches!(self.scene(),Scene::Video|Scene::Gallery){
            self.rewinding=true;
            if self.scene()==Scene::Video{self.effects.push(Effect::StopVideo);}
        }
    }
    pub fn xml_loaded(&mut self,paths:Vec<String>){
        if let Some(path)=self.gallery.install(paths){self.effects.push(Effect::Image(path));}
    }
    pub fn activate(&mut self,hit:Hit,x:f64){
        let scene=self.scene();
        match hit{
            Hit::Accept if scene==Scene::Launcher=>{self.launched=true;self.effects.extend([Effect::Fullscreen,Effect::Play(INTRO)]);}
            Hit::Decline if scene==Scene::Launcher=>self.effects.push(Effect::Quit),
            Hit::Quit if matches!(scene,Scene::Welcome|Scene::Wallpapers|Scene::Videos|Scene::Screens)=>self.effects.push(Effect::Quit),
            Hit::Website if matches!(scene,Scene::Welcome|Scene::Wallpapers|Scene::Videos|Scene::Screens)=>self.effects.push(Effect::Open(WEBSITE)),
            Hit::Wallpapers|Hit::Videos|Hit::Screens if matches!(scene,Scene::Welcome|Scene::Wallpapers|Scene::Videos|Scene::Screens)=>{
                self.frame=match hit{Hit::Wallpapers=>76,Hit::Videos=>86,_=>96};self.playing=true;self.effects.push(Effect::Sound(2));
            }
            Hit::Wallpaper(id) if scene==Scene::Wallpapers=>{if let Some(p)=id.checked_sub(1).and_then(|i|WALLPAPERS.get(usize::from(i))){self.effects.push(Effect::Open(p));}}
            Hit::Trailer(id) if scene==Scene::Videos&&trailer(id).is_some()=>{self.video=id;self.playing=true;self.effects.extend([Effect::Music(false),Effect::Sound(154)]);}
            Hit::Thumbnail(id) if scene==Scene::Screens=>{if self.gallery.select(id){self.playing=true;}}
            Hit::Close=>self.close_media(),
            Hit::Previous if scene==Scene::Gallery=>{if let Some(p)=self.gallery.previous(){self.effects.push(Effect::Image(p));}}
            Hit::Next if scene==Scene::Gallery=>{if let Some(p)=self.gallery.next(){self.effects.push(Effect::Image(p));}}
            Hit::PlayPause if scene==Scene::Video=>self.effects.push(Effect::ToggleVideo),
            Hit::Seek if scene==Scene::Video=>self.effects.push(Effect::Seek(((x-204.)/872.).clamp(0.,1.))),
            _=>{}
        }
    }
    pub fn tick(&mut self){
        if !self.launched{return;}
        if self.rewinding{self.frame-=1;self.enter();}
        else if self.playing&&self.frame<105{self.frame+=1;self.enter();}
        if self.frame>=56{self.background.advance(1000./24.);}
        if matches!(self.scene(),Scene::Gallery|Scene::Video){self.controls.tick();}
    }
    pub fn advance(&mut self,ms:f64){self.advance_when_ready(ms,true);}
    /// Keep timeline transitions on their last complete frame until the next
    /// frame's required artwork has arrived. Static scenes are never gated.
    pub fn advance_when_ready(&mut self,ms:f64,transition_ready:bool){
        if self.scene()==Scene::Transition&&!transition_ready{return;}
        if !ms.is_finite()||ms<=0.{return;}
        self.credit+=ms.min(250.);
        while self.credit>=1000./24.{self.credit-=1000./24.;self.tick();}
    }
    fn enter(&mut self){
        match self.frame{
            2=>self.effects.push(Effect::Sound(52)),
            56=>{self.background=Background::default();self.effects.push(Effect::Music(true));}
            75|81|85=>self.playing=false,
            91=>{self.playing=false;self.rewinding=false;self.effects.push(Effect::Music(true));}
            101=>{self.playing=false;self.rewinding=false;}
            95=>{self.playing=false;self.controls=Controls::default();if let Some(p)=trailer(self.video){self.effects.push(Effect::Play(p));}}
            105=>{self.playing=false;self.controls=Controls{y:1346.,..Controls::default()};self.effects.push(Effect::ReadXml(SCREEN_XML));}
            _=>{}
        }
    }
}


#[cfg(test)]
mod transition_tests {
    use super::*;
    #[test]
    fn menu_transition_waits_for_current_frame_artwork() {
        let mut app=App::default();
        app.launched=true;
        app.frame=75;
        app.activate(Hit::Videos,0.);
        assert_eq!(app.scene(),Scene::Transition);
        assert_eq!(app.frame,86);
        app.advance_when_ready(1000./24.,false);
        assert_eq!(app.frame,86);
        for expected in 87..=91 {
            app.advance_when_ready(1000./24.,true);
            assert_eq!(app.frame,expected);
        }
        assert_eq!(app.scene(),Scene::Videos);
    }

    #[test]
    fn readiness_gate_does_not_block_non_transition_scenes() {
        let mut app=App::default();
        app.launched=true;
        app.frame=91;
        app.playing=true;
        app.advance_when_ready(1000./24.,false);
        assert_eq!(app.frame,92);
        assert_eq!(app.scene(),Scene::Transition);
    }
}

#[cfg(test)]
mod steady_menu_tests {
    use super::*;
    #[test]
    fn every_menu_pair_skips_original_blank_frames_and_retains_background(){
        for start in [75,81,91,101]{
            for (hit,target) in [(Hit::Wallpapers,81),(Hit::Videos,91),(Hit::Screens,101)]{
                let mut app=App::default();app.launched=true;app.frame=start;
                app.background.frame=110;
                assert_eq!(app.steady_menu_frame(hit),Some(target));
                assert!(app.activate_steady_menu(hit));
                assert_eq!(app.frame,target);assert!(!app.playing);assert!(!app.rewinding);
                assert_eq!(app.background.frame,110);
                for _ in 0..8{app.tick();assert_eq!(app.frame,target);}
            }
        }
    }
    #[test]
    fn original_timeline_and_media_fade_remain_available(){
        let mut a=App::default();a.launched=true;a.frame=75;
        a.activate(Hit::Videos,0.);assert_eq!(a.frame,86);
        for _ in 0..5{a.tick();}assert_eq!(a.frame,91);
        a.activate(Hit::Trailer(4),0.);
        for expected in 92..=95{a.tick();assert_eq!(a.frame,expected);}
        assert!(!a.activate_steady_menu(Hit::Screens));
    }
}
