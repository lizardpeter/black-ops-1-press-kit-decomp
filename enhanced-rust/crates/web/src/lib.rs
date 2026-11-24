#![cfg(target_arch="wasm32")]
use presskit_core::{App,Effect,Hit,Scene,Stage,flash_time};
use presskit_renderer::{Layer,Renderer};
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
    photo:String,layered:bool,serial:u64,control_y:f32,
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
        let renderer=Renderer::new(&device,render_format,VS,FS,"vertexMain","fragmentMain");
        Ok(Self{app:App::default(),canvas,surface,device,queue,config,renderer,images:HashMap::new(),video:None,photo:String::new(),layered,serial:0,control_y:-1.})
    }
    pub fn resize(&mut self,w:u32,h:u32){let w=w.max(1);let h=h.max(1);if (w,h)!=(self.config.width,self.config.height){self.canvas.set_width(w);self.canvas.set_height(h);self.config.width=w;self.config.height=h;self.surface.configure(&self.device,&self.config);}}
    pub fn advance(&mut self,ms:f64){self.app.advance(ms);}
    pub fn scene(&self)->String{format!("{:?}",self.app.scene())}
    pub fn frame(&self)->u16{self.app.frame}
    pub fn controls_y(&self)->f64{self.app.controls.y}
    pub fn gallery_counter(&self)->String{self.app.gallery.counter()}
    pub fn media_time(&self,position:f64,duration:f64)->String{format!("{}/{}",flash_time(position),flash_time(duration))}
    pub fn pointer(&mut self){self.app.controls.activity();}
    pub fn click(&mut self,x:f64,y:f64){self.pointer();if let Some((x,y))=Stage::fit(self.config.width.into(),self.config.height.into()).point(x,y){let h=self.app.hit(x,y);self.app.activate(h,x);}}
    pub fn key(&mut self,key:&str){match key{
        "Enter" if self.app.scene()==Scene::Launcher=>self.app.activate(Hit::Accept,0.),
        "Escape" if self.app.scene()==Scene::Intro=>self.media_complete(),
        "Escape"=>self.app.activate(Hit::Close,0.),"ArrowLeft"=>self.app.activate(Hit::Previous,0.),"ArrowRight"=>self.app.activate(Hit::Next,0.)," "=>self.app.activate(Hit::PlayPause,0.),_=>{}}}
    pub fn media_complete(&mut self){self.app.media_complete();self.video=None;}
    pub fn xml_loaded(&mut self,paths:Array){self.app.xml_loaded(paths.iter().filter_map(|v|v.as_string()).filter(|v|v.starts_with("payload/screens/")&&!v.contains("..")).collect());}
    pub fn effects(&mut self)->Array{
        let out=Array::new();for e in self.app.effects(){let p=match e{
            Effect::Fullscreen=>pair("fullscreen",""),Effect::Quit=>pair("quit",""),Effect::Play(p)=>{self.video=None;pair("play",p)},Effect::StopVideo=>{self.video=None;pair("stop","")},Effect::ToggleVideo=>pair("toggle",""),Effect::Seek(v)=>pair("seek",&v.to_string()),Effect::Music(on)=>pair("music",if on{"1"}else{"0"}),Effect::Sound(id)=>pair("sound",&id.to_string()),Effect::Open(p)=>pair("open",p),Effect::ReadXml(p)=>pair("xml",p),Effect::Image(p)=>{self.photo=p.clone();pair("image",&p)}};out.push(&p);}out
    }
    fn paths(&self)->Vec<String>{
        let scene=self.app.scene();let mut p=vec![];
        match scene{Scene::Launcher=>p.push("art/launcher/1.png".into()),Scene::Gallery=>{if !self.photo.is_empty(){p.push(self.photo.clone());}},Scene::Intro|Scene::Video=>{},_=>{
            if self.layered&&self.app.frame>=56{p.push(format!("art/background/{}.png",self.app.background.frame));}
            p.push(format!("art/{}/{}.png",if self.layered{"foreground"}else{"presskit"},self.app.frame));}}
        if scene==Scene::Video{p.push("art/video_controls/1.png".into());}else if scene==Scene::Gallery{p.push("art/gallery_controls/1.png".into());}p
    }
    pub fn needed(&self)->Array{self.paths().into_iter().filter(|p|!self.images.contains_key(p)).map(JsValue::from).collect()}
    pub fn upload(&mut self,path:String,bitmap:ImageBitmap){
        let rect=if path=="art/launcher/1.png"{[390.,210.,500.,300.]}else if path.contains("_controls/"){[200.,self.app.controls.y as f32,880.,90.]}else{[0.,0.,1280.,720.]};
        let layer=self.renderer.external_layer(&self.device,&self.queue,wgpu::ExternalImageSource::ImageBitmap(bitmap),rect);
        self.serial+=1;self.images.insert(path,(layer,self.serial));
        if self.images.len()>48{let used=self.paths();if let Some(key)=self.images.iter().filter(|(k,_)|!used.contains(k)).min_by_key(|(_,(_,s))|*s).map(|(k,_)|k.clone()){self.images.remove(&key);}}
    }
    pub fn draw(&mut self,video:HtmlVideoElement)->Result<(),JsValue>{
        let media=matches!(self.app.scene(),Scene::Intro|Scene::Video)&&!self.app.playing;
        let paused=video.paused();let progress=if video.duration().is_finite()&&video.duration()>0.{(video.current_time()/video.duration()).clamp(0.,1.) as f32}else{0.};
        if media&&video.ready_state()>=2&&video.video_width()>0{
            let source=wgpu::ExternalImageSource::HTMLVideoElement(video.clone());
            if self.video.as_ref().is_some_and(|v|v.texture.width()==source.width()&&v.texture.height()==source.height()){
                Renderer::copy_external(&self.queue,self.video.as_ref().unwrap(),source);
            }else{self.video=Some(self.renderer.external_layer(&self.device,&self.queue,source,[0.,0.,1280.,720.]));}
        }
        let vectors=if self.app.scene()==Scene::Video{
            let mut v=self.renderer.video_vectors(&self.device,&self.queue,self.app.controls.y as f32,paused,progress);
            let text=self.media_time(video.current_time(),if video.duration().is_finite(){video.duration()}else{0.});
            if let Some(t)=self.renderer.counter(&self.device,&self.queue,&text,self.app.controls.y as f32,false){v.push(t);}v
        }else if self.app.scene()==Scene::Gallery{self.renderer.gallery_vectors(&self.device,&self.queue,self.app.controls.y as f32,&self.app.gallery.counter())}else{vec![]};
        let paths=self.paths();self.serial+=1;
        for p in &paths{if let Some((layer,used))=self.images.get_mut(p){*used=self.serial;if p.contains("_controls/")&&self.control_y!=self.app.controls.y as f32{self.renderer.reposition(&self.device,layer,[200.,self.app.controls.y as f32,880.,90.]);}}}
        self.control_y=self.app.controls.y as f32;
        let frame=match self.surface.get_current_texture(){Ok(f)=>f,Err(wgpu::SurfaceError::Lost|wgpu::SurfaceError::Outdated)=>{self.surface.configure(&self.device,&self.config);return Ok(());},Err(e)=>return Err(error(e))};
        let view=frame.texture.create_view(&wgpu::TextureViewDescriptor{format:Some(self.config.format.add_srgb_suffix()),..Default::default()});let mut encoder=self.device.create_command_encoder(&Default::default());
        let stage=Stage::fit(self.config.width.into(),self.config.height.into());let mut layers=vec![];
        if media{if let Some(v)=&self.video{layers.push(v);}}
        for p in paths{if let Some((layer,_))=self.images.get(&p){layers.push(layer);}}
        layers.extend(vectors.iter());
        self.renderer.render(&mut encoder,&view,&layers,[stage.x as f32,stage.y as f32,(1280.*stage.scale) as f32,(720.*stage.scale) as f32]);
        self.queue.submit([encoder.finish()]);frame.present();Ok(())
    }
}
