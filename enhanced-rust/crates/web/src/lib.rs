#![cfg(target_arch="wasm32")]
use presskit_core::{App,Effect,Hit,Scene,Stage,flash_time};
use presskit_renderer::{Layer,Renderer};
use presskit_renderer::background::{BackgroundArt,BackgroundTimeline};
use presskit_renderer::foreground::{ForegroundArt,ForegroundTimeline};
use presskit_renderer::control_pack::{ControlPack,ControlCache,ControlVisual};
use presskit_core::assets::AssetPlan;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement,HtmlVideoElement,ImageBitmap};
use js_sys::Array;
use std::collections::HashMap;
const VS:&str=include_str!("../../../shaders/generated/vertex.wgsl");
const FS:&str=include_str!("../../../shaders/generated/fragment.wgsl");
fn error(e:impl std::fmt::Debug)->JsValue{JsValue::from_str(&format!("{e:?}"))}
fn pair(kind:&str,value:&str)->Array{Array::of2(&kind.into(),&value.into())}
#[wasm_bindgen]
pub struct WebPlayer{
    app:App,canvas:HtmlCanvasElement,surface:wgpu::Surface<'static>,device:wgpu::Device,queue:wgpu::Queue,
    config:wgpu::SurfaceConfiguration,renderer:Renderer,images:HashMap<String,(Layer,u64)>,video:Option<Layer>,
    photo:String,layered:bool,serial:u64,control_y:f32,video_stamp:f64,last_foreground:Option<String>,background:Option<BackgroundArt>,foreground:Option<ForegroundArt>,pack:Option<ControlPack>,controls:ControlCache,mouse:Option<(f64,f64)>,pressed:bool,
}
#[wasm_bindgen]
impl WebPlayer{
    pub async fn create(canvas:HtmlCanvasElement,layered:bool)->Result<WebPlayer,JsValue>{
        let instance=wgpu::Instance::new(&wgpu::InstanceDescriptor{backends:wgpu::Backends::BROWSER_WEBGPU,..Default::default()});
        let surface=instance.create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone())).map_err(error)?;
        let adapter=instance.request_adapter(&wgpu::RequestAdapterOptions{compatible_surface:Some(&surface),..Default::default()}).await.ok_or_else(||JsValue::from_str("WebGPU adapter unavailable"))?;
        let (device,queue)=adapter.request_device(&wgpu::DeviceDescriptor::default(),None).await.map_err(error)?;
        let mut config=surface.get_default_config(&adapter,canvas.width().max(1),canvas.height().max(1)).ok_or_else(||JsValue::from_str("surface configuration unavailable"))?;
        let render_format=config.format.add_srgb_suffix();
        if render_format!=config.format{config.view_formats.push(render_format);}
        surface.configure(&device,&config);
        let renderer=Renderer::new(&device,&queue,render_format,VS,FS,"vertexMain","fragmentMain");
        Ok(Self{app:App::default(),canvas,surface,device,queue,config,renderer,images:HashMap::new(),video:None,photo:String::new(),layered,serial:0,control_y:-1.,video_stamp:-1.,last_foreground:None,background:None,foreground:None,pack:None,controls:ControlCache::default(),mouse:None,pressed:false})
    }
    pub fn resize(&mut self,w:u32,h:u32){let w=w.max(1);let h=h.max(1);if (w,h)!=(self.config.width,self.config.height){self.canvas.set_width(w);self.canvas.set_height(h);self.config.width=w;self.config.height=h;self.surface.configure(&self.device,&self.config);}}
    pub fn advance(&mut self,ms:f64){self.app.advance(ms);}
    pub fn scene(&self)->String{format!("{:?}",self.app.scene())}
    pub fn frame(&self)->u16{self.app.frame}
    pub fn controls_y(&self)->f64{self.app.controls.y}
    pub fn gallery_counter(&self)->String{self.app.gallery.counter()}
    pub fn media_time(&self,position:f64,duration:f64)->String{format!("{}/{}",flash_time(position),flash_time(duration))}
    pub fn pointer(&mut self){self.app.controls.activity();}
    pub fn pointer_at(&mut self,x:f64,y:f64,pressed:bool){self.pointer();self.mouse=Stage::fit(self.config.width.into(),self.config.height.into()).point(x,y);self.pressed=pressed;}
    pub fn install_background(&mut self,bytes:&[u8])->Result<(),JsValue>{self.background=Some(BackgroundArt::new(BackgroundTimeline::read(bytes).map_err(error)?));Ok(())}
    pub fn install_foreground(&mut self,bytes:&[u8])->Result<(),JsValue>{self.foreground=Some(ForegroundArt::new(ForegroundTimeline::read(bytes).map_err(error)?));Ok(())}
    pub fn install_menu(&mut self,bytes:&[u8])->Result<(),JsValue>{self.foreground.as_mut().ok_or_else(||JsValue::from_str("foreground missing"))?.install_menu(bytes).map_err(error)}
    pub fn install_controls(&mut self,bytes:&[u8])->Result<(),JsValue>{self.pack=Some(ControlPack::read(bytes).map_err(error)?);self.controls.clear();Ok(())}
    pub fn click(&mut self,x:f64,y:f64){self.pointer();if !self.input_ready(){return;}if let Some((x,y))=Stage::fit(self.config.width.into(),self.config.height.into()).point(x,y){let h=self.app.hit(x,y);self.app.activate(h,x);}}
    pub fn key(&mut self,key:&str){match key{
        "Enter" if self.app.scene()==Scene::Launcher=>self.app.activate(Hit::Accept,0.),
        "Escape" if self.app.scene()==Scene::Intro=>self.media_complete(),
        "Escape"=>self.app.activate(Hit::Close,0.),"ArrowLeft" if self.input_ready()=>self.app.activate(Hit::Previous,0.),"ArrowRight" if self.input_ready()=>self.app.activate(Hit::Next,0.)," "=>self.app.activate(Hit::PlayPause,0.),_=>{}}}
    pub fn media_complete(&mut self){self.app.media_complete();self.video=None;}
    pub fn xml_loaded(&mut self,paths:Array){if self.app.scene()!=Scene::Gallery||self.app.rewinding{return;}self.app.xml_loaded(paths.iter().filter_map(|v|v.as_string()).filter(|v|v.starts_with("payload/screens/")&&!v.contains("..")).collect());}
    pub fn effects(&mut self)->Array{
        let out=Array::new();for e in self.app.effects(){let p=match e{
            Effect::Fullscreen=>pair("fullscreen",""),Effect::Quit=>pair("quit",""),Effect::Play(p)=>{self.video=None;pair("play",p)},Effect::StopVideo=>{self.video=None;pair("stop","")},Effect::ToggleVideo=>pair("toggle",""),Effect::Seek(v)=>pair("seek",&v.to_string()),Effect::Music(on)=>pair("music",if on{"1"}else{"0"}),Effect::Sound(id)=>pair("sound",&id.to_string()),Effect::Open(p)=>pair("open",p),Effect::ReadXml(p)=>{self.photo.clear();pair("xml",p)},Effect::Image(p)=>{self.photo=p.clone();pair("image",&p)}};out.push(&p);}out
    }
    fn foreground_on(&self)->bool{self.layered&&self.foreground.is_some()&&matches!(self.app.scene(),Scene::Transition|Scene::Welcome|Scene::Wallpapers|Scene::Videos|Scene::Screens)}
    fn plan(&self)->AssetPlan{let mut p=AssetPlan::for_app(&self.app,self.layered,&self.photo);if let Some(bg)=&self.background{for i in bg.missing(){p.ahead.insert(0,format!("art/background-source/{i}.jpg"));}}if let Some(fg)=&self.foreground{p.current.retain(|s|!s.starts_with("art/foreground/"));p.ahead.retain(|s|!s.starts_with("art/foreground/"));if self.foreground_on(){for i in fg.missing_frame(self.app.frame){p.current.push(format!("art/foreground-source/{}",fg.filename(i)));}}for i in fg.missing(){p.ahead.push(format!("art/foreground-source/{}",fg.filename(i)));}}if self.pack.is_some(){p.current.retain(|s|!s.contains("_controls/"));p.ahead.retain(|s|!s.contains("_controls/"));}p}
    fn paths(&self)->Vec<String>{self.plan().current}
    pub fn needed(&self)->Array{self.paths().into_iter().filter(|p|!self.images.contains_key(p)).map(JsValue::from).collect()}
    pub fn prefetch(&self)->Array{self.plan().ahead.into_iter().filter(|p|!self.images.contains_key(p)).map(JsValue::from).collect()}
    pub fn ready(&self)->bool{self.paths().iter().all(|p|self.images.contains_key(p))&&(!self.layered||self.app.frame<56||matches!(self.app.scene(),Scene::Gallery|Scene::Video)||self.background.as_ref().is_some_and(|b|b.ready()))}
    fn input_ready(&self)->bool{
        let scene=self.app.scene();if scene==Scene::Gallery&&self.photo.is_empty(){return false;}
        self.ready()
    }
    pub fn cache_bytes(&self)->f64{self.images.values().map(|(l,_)|u64::from(l.texture.width())*u64::from(l.texture.height())*4).sum::<u64>() as f64}
    fn visible_paths(&mut self)->Vec<String>{
        let mut paths=self.paths();
        for p in &mut paths{
            let last=if p.starts_with("art/foreground/")||p.starts_with("art/presskit/"){&mut self.last_foreground}else{continue;};
            if self.images.contains_key(p){*last=Some(p.clone());}else if let Some(previous)=last.as_ref().filter(|s|self.images.contains_key(*s)){*p=previous.clone();}
        }paths
    }
    pub fn upload(&mut self,path:String,bitmap:ImageBitmap){
        let rect=if path=="art/launcher/1.png"{[390.,210.,500.,300.]}else if path.contains("_controls/"){[200.,self.app.controls.y as f32,880.,90.]}else{[0.,0.,1280.,720.]};
        let layer=self.renderer.external_layer(&self.device,&self.queue,wgpu::ExternalImageSource::ImageBitmap(bitmap),rect);
        if let Some(i)=path.strip_prefix("art/background-source/").and_then(|s|s.strip_suffix(".jpg")).and_then(|s|s.parse::<usize>().ok()){if let Some(bg)=&mut self.background{let _=bg.install(i,layer);}return;}
        if let Some(i)=path.strip_prefix("art/foreground-source/").and_then(|s|s.strip_suffix(".png").or_else(||s.strip_suffix(".jpg"))).and_then(|s|s.parse::<usize>().ok()){if let Some(fg)=&mut self.foreground{let _=fg.install(i,layer);}return;}
        self.serial+=1;self.images.insert(path,(layer,self.serial));
        while self.images.len()>32||self.cache_bytes()>96.*1024.*1024.{let used=self.paths();let victim=self.images.iter().filter(|(k,_)|!used.contains(k)&&self.last_foreground.as_ref()!=Some(*k)).min_by_key(|(_,(_,s))|*s).map(|(k,_)|k.clone());if let Some(key)=victim{self.images.remove(&key);}else{break;}}
    }
    pub fn draw(&mut self,video:HtmlVideoElement,stamp:f64)->Result<(),JsValue>{
        let media=matches!(self.app.scene(),Scene::Intro|Scene::Video)&&!self.app.playing;
        let paused=video.paused();let progress=if video.duration().is_finite()&&video.duration()>0.{(video.current_time()/video.duration()).clamp(0.,1.) as f32}else{0.};
        if media&&video.ready_state()>=2&&video.video_width()>0&&(self.video.is_none()||stamp!=self.video_stamp){
            self.video_stamp=stamp;
            let source=wgpu::ExternalImageSource::HTMLVideoElement(video.clone());
            if self.video.as_ref().is_some_and(|v|v.texture.width()==source.width()&&v.texture.height()==source.height()){
                Renderer::copy_external(&self.queue,self.video.as_ref().unwrap(),source);
            }else{self.video=Some(self.renderer.external_layer(&self.device,&self.queue,source,[0.,0.,1280.,720.]));}
        }
        if matches!(self.app.scene(),Scene::Video|Scene::Gallery){
            let gallery=self.app.scene()==Scene::Gallery;let text=if gallery{self.app.gallery.counter()}else{self.media_time(video.current_time(),if video.duration().is_finite(){video.duration()}else{0.})};
            let hit=self.mouse.map(|(x,y)|self.app.hit(x,y)).unwrap_or(Hit::None);let states=presskit_core::control_states(hit,self.pressed);
            self.controls.update(self.pack.as_ref(),&self.renderer,&self.device,&self.queue,ControlVisual{y:self.app.controls.y as f32,gallery,paused,progress,text,states});
        }else{self.controls.clear();}
        let fg_on=self.foreground_on();if fg_on{self.foreground.as_mut().unwrap().update(&self.renderer,&self.queue,self.app.frame);}
        if fg_on{let selected=match self.mouse.map(|(x,y)|self.app.hit(x,y)){Some(Hit::Trailer(id)|Hit::Thumbnail(id))=>id,_=>0};self.foreground.as_mut().unwrap().update_menu(&self.renderer,&self.device,&self.queue,selected,self.pressed);}
        let paths=self.visible_paths();self.serial+=1;
        for p in &paths{if let Some((layer,used))=self.images.get_mut(p){*used=self.serial;if p.contains("_controls/")&&self.control_y!=self.app.controls.y as f32{self.renderer.reposition(&self.device,layer,[200.,self.app.controls.y as f32,880.,90.]);}}}
        self.control_y=self.app.controls.y as f32;
        let frame=match self.surface.get_current_texture(){Ok(f)=>f,Err(wgpu::SurfaceError::Lost|wgpu::SurfaceError::Outdated)=>{self.surface.configure(&self.device,&self.config);return Ok(());},Err(e)=>return Err(error(e))};
        let view=frame.texture.create_view(&wgpu::TextureViewDescriptor{format:Some(self.config.format.add_srgb_suffix()),..Default::default()});let mut encoder=self.device.create_command_encoder(&Default::default());
        let stage=Stage::fit(self.config.width.into(),self.config.height.into());let mut layers=vec![];
        if self.layered&&self.app.frame>=56&&!matches!(self.app.scene(),Scene::Video|Scene::Gallery){if let Some(bg)=&mut self.background{bg.update(&self.renderer,&self.device,&self.queue,self.app.background.frame);layers.extend(bg.layers(self.app.background.frame));}}
        if fg_on{layers.extend(self.foreground.as_ref().unwrap().layers());}
        if media{if let Some(v)=&self.video{layers.push(v);}}
        for p in paths{if let Some((layer,_))=self.images.get(&p){layers.push(layer);}}
        layers.extend(self.controls.layers().iter());
        self.renderer.render(&mut encoder,&view,&layers,[stage.x as f32,stage.y as f32,(1280.*stage.scale) as f32,(720.*stage.scale) as f32]);
        self.queue.submit([encoder.finish()]);frame.present();Ok(())
    }
}


/// Run the actual shared application logic inside the compiled WASM module,
/// without creating a GPU surface. This is not a browser graphics test.
#[wasm_bindgen]
pub fn wasm_core_smoke()->bool{
    let mut app=App::default();app.activate(Hit::Accept,0.);
    if !app.effects().contains(&Effect::Play(presskit_core::INTRO)){return false;}
    app.media_complete();for _ in 0..74{app.tick();}
    if app.scene()!=Scene::Welcome{return false;}
    app.activate(Hit::Videos,0.);for _ in 0..5{app.tick();}
    app.activate(Hit::Trailer(4),0.);for _ in 0..4{app.tick();}
    if app.scene()!=Scene::Video||!app.effects().contains(&Effect::Play(presskit_core::trailer(4).unwrap())){return false;}
    app.media_complete();for _ in 0..4{app.tick();}
    app.activate(Hit::Screens,0.);for _ in 0..5{app.tick();}
    app.activate(Hit::Thumbnail(20),0.);for _ in 0..4{app.tick();}
    app.xml_loaded((1..=20).map(|i|format!("payload/screens/{i}.jpg")).collect());
    let plan=AssetPlan::for_app(&app,true,"payload/screens/20.jpg");
    app.activate(Hit::Previous,0.);
    app.scene()==Scene::Gallery&&app.gallery.counter()=="19 / 20"&&plan.ahead.contains(&"payload/screens/19.jpg".into())
}

 
/// Validate the actual runtime asset table in WASM without requiring a GPU adapter.
#[wasm_bindgen]
pub fn wasm_validate_foreground(timeline:&[u8], menu:&[u8])->Result<u32,JsValue>{
    let t=ForegroundTimeline::read(timeline).map_err(error)?;
    let jpeg_count=(0..t.count()).filter(|&i|t.filename(i).ends_with(".jpg")).count() as u32;
    let mut art=ForegroundArt::new(t);
    if art.needs_menu(){art.install_menu(menu).map_err(error)?;}
    Ok(jpeg_count)
}
