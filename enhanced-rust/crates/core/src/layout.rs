use crate::Scene;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Hit { None, Accept, Decline, Wallpapers, Videos, Screens, Website, Quit, Wallpaper(u8), Trailer(u8), Thumbnail(u8), Close, Previous, Next, PlayPause, Seek }
#[derive(Clone,Copy,Debug)]
pub struct Stage {pub scale:f64,pub x:f64,pub y:f64}
impl Stage {
    pub fn fit(w:f64,h:f64)->Self{
        let scale=(w/1280.).min(h/720.).max(0.);
        Self{scale,x:(w-1280.*scale)/2.,y:(h-720.*scale)/2.}
    }
    pub fn point(self,x:f64,y:f64)->Option<(f64,f64)>{
        if self.scale<=0. {return None;}
        let (x,y)=((x-self.x)/self.scale,(y-self.y)/self.scale);
        (x>=0.&&x<1280.&&y>=0.&&y<720.).then_some((x,y))
    }
}
fn inside(x:f64,y:f64,l:f64,t:f64,w:f64,h:f64)->bool{x>=l&&x<l+w&&y>=t&&y<t+h}
pub fn hit_launcher(x:f64,y:f64)->Hit{
    if inside(x,y,439.,409.,188.,54.){Hit::Accept}
    else if inside(x,y,649.,409.,188.,54.){Hit::Decline}else{Hit::None}
}
pub fn hit_menu(scene:Scene,x:f64,y:f64)->Hit{
    if !matches!(scene,Scene::Welcome|Scene::Wallpapers|Scene::Videos|Scene::Screens){return Hit::None;}
    if inside(x,y,1091.5,26.5,16.05,16.05){return Hit::Quit;}
    let sx=97784./65536.;let sy=228319./65536.;
    let nx=233.+628.*sx-937.;
    for (top,hit) in [(138.,Hit::Wallpapers),(260.,Hit::Videos),(382.,Hit::Screens)]{
        if inside(x,y,nx,top,126.*sx,31.*sy){return hit;}
    }
    if inside(x,y,233.,542.,188.,54.){return Hit::Website;}
    match scene {
        Scene::Wallpapers=>for id in 1..=8u8 {
            let l=526.85+[6.,154.,301.,449.][usize::from((id-1)/2)];
            let t=if id%2==1{487.}else{523.};
            if inside(x,y,l,t,87.,29.){return Hit::Wallpaper(id);}
        },
        Scene::Videos=>for (id,l,t) in [(1,533.,208.),(2,712.,208.),(4,891.,208.),(5,533.,381.),(6,712.,381.)]{
            if inside(x,y,l,t,172.,96.){return Hit::Trailer(id);}
        },
        Scene::Screens=>for id in 1..=20u8{
            if inside(x,y,533.+107.*f64::from((id-1)%5),158.+95.*f64::from((id-1)/5),102.,90.){return Hit::Thumbnail(id);}
        },_=>{}
    } Hit::None
}
pub fn hit_media(scene:Scene,x:f64,y:f64,cy:f64)->Hit{
    if !matches!(scene,Scene::Gallery|Scene::Video){return Hit::None;}
    if inside(x,y,961.,cy,102.95,31.){return Hit::Close;}
    if scene==Scene::Gallery{
        if inside(x,y,215.,cy,112.,31.){return Hit::Previous;}
        if inside(x,y,365.,cy,114.6,31.){return Hit::Next;}
    }else{
        if inside(x,y,215.,cy,66.,31.){return Hit::PlayPause;}
        if inside(x,y,204.,cy+48.,872.,24.){return Hit::Seek;}
    }
    Hit::None
}
