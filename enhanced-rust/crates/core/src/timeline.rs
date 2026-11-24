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
            Hit::Seek if scene==Scene::Video=>self.effects.push(Effect::Seek(((x-241.)/785.).clamp(0.,1.))),
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
    pub fn advance(&mut self,ms:f64){
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
