use presskit_core::{App,Effect,Hit,Scene,Stage};
use presskit_renderer::{Renderer,Layer};
use std::{sync::Arc,path::PathBuf,time::Instant};
use winit::{application::ApplicationHandler,event::{WindowEvent,ElementState,MouseButton},event_loop::{ActiveEventLoop,EventLoop},window::{Window,WindowId}};
const VERTEX:&str=include_str!("../../../shaders/generated/vertex.wgsl");
const FRAGMENT:&str=include_str!("../../../shaders/generated/fragment.wgsl");
struct Gpu{
    window:Arc<Window>,surface:wgpu::Surface<'static>,device:wgpu::Device,queue:wgpu::Queue,config:wgpu::SurfaceConfiguration,renderer:Renderer,
    art:Option<Layer>,bg:Option<Layer>,last_art:String,last_bg:u16
}
impl Gpu{
    async fn new(window:Arc<Window>)->Self{
        let instance=wgpu::Instance::default();let surface=instance.create_surface(window.clone()).expect("surface");
        let adapter=instance.request_adapter(&wgpu::RequestAdapterOptions{compatible_surface:Some(&surface),..Default::default()}).await.expect("GPU adapter");
        let (device,queue)=adapter.request_device(&wgpu::DeviceDescriptor::default(),None).await.expect("GPU device");
        let size=window.inner_size();let mut config=surface.get_default_config(&adapter,size.width.max(1),size.height.max(1)).expect("surface config");
        let caps=surface.get_capabilities(&adapter);
        if let Some(f)=caps.formats.iter().find(|f|f.is_srgb()){config.format=*f;}
        surface.configure(&device,&config);
        let renderer=Renderer::new(&device,config.format,VERTEX,FRAGMENT,"vertexMain","fragmentMain");
        Self{window,surface,device,queue,config,renderer,art:None,bg:None,last_art:String::new(),last_bg:0}
    }
    fn resize(&mut self,w:u32,h:u32){if w>0&&h>0{self.config.width=w;self.config.height=h;self.surface.configure(&self.device,&self.config);}}
    fn image(&self,path:&std::path::Path,rect:[f32;4])->Option<Layer>{
        let img=image::open(path).map_err(|e|eprintln!("{}: {e}",path.display())).ok()?.into_rgba8();
        Some(self.renderer.layer(&self.device,&self.queue,img.as_raw(),img.width(),img.height(),rect))
    }
    fn draw(&mut self,root:&std::path::Path,app:&App,photo:&Option<String>){
        let layered=root.join("art/foreground/75.png").is_file()&&root.join("art/background/221.png").is_file();
        let (path,rect)=match app.scene(){
            Scene::Launcher=>("art/launcher/1.png".to_string(),[390.,210.,500.,300.]),
            Scene::Gallery=>(photo.clone().unwrap_or_default(),[0.,0.,1280.,720.]),
            Scene::Intro|Scene::Video=>(String::new(),[0.,0.,1280.,720.]),
            _=>(format!("art/{}/{}.png",if layered{"foreground"}else{"presskit"},app.frame),[0.,0.,1280.,720.])
        };
        if path!=self.last_art{self.art=if path.is_empty(){None}else{self.image(&root.join(&path),rect)};self.last_art=path;}
        let bg_on=layered&&app.frame>=56&&!matches!(app.scene(),Scene::Gallery|Scene::Video);
        if bg_on&&app.background.frame!=self.last_bg{
            self.bg=self.image(&root.join(format!("art/background/{}.png",app.background.frame)),[0.,0.,1280.,720.]);self.last_bg=app.background.frame;
        }
        let size=self.window.inner_size();if size.width==0||size.height==0{return;}
        let frame=match self.surface.get_current_texture(){Ok(f)=>f,Err(wgpu::SurfaceError::Lost|wgpu::SurfaceError::Outdated)=>{self.resize(size.width,size.height);return;},Err(e)=>{eprintln!("surface: {e}");return;}};
        let view=frame.texture.create_view(&Default::default());
        let mut encoder=self.device.create_command_encoder(&Default::default());
        let stage=Stage::fit(f64::from(size.width),f64::from(size.height));
        let mut layers=vec![];if bg_on{if let Some(b)=&self.bg{layers.push(b);}}if let Some(a)=&self.art{layers.push(a);}
        self.renderer.render(&mut encoder,&view,&layers,[stage.x as f32,stage.y as f32,(1280.*stage.scale) as f32,(720.*stage.scale) as f32]);
        self.queue.submit([encoder.finish()]);frame.present();
    }
}
struct Host{root:PathBuf,app:App,gpu:Option<Gpu>,pointer:(f64,f64),last:Instant,photo:Option<String>}
impl Host{
    fn dispatch(&mut self,event_loop:&ActiveEventLoop){
        for effect in self.app.effects(){match effect{
            Effect::Quit=>event_loop.exit(),
            Effect::Fullscreen=>if let Some(g)=&self.gpu{g.window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));},
            Effect::Image(p)=>self.photo=Some(p),
            Effect::ReadXml(_)=>{
                // Temporary manifest reader for the known original XML structure.
                if let Ok(xml)=std::fs::read_to_string(self.root.join(presskit_core::SCREEN_XML)){
                    let paths=xml.split("<image>").skip(1).filter_map(|s|s.split_once("</image>").map(|x|x.0.to_owned())).filter(|s|s.starts_with("payload/screens/")&&!s.contains("..")).collect();
                    self.app.xml_loaded(paths);
                }
            }
            Effect::Play(p)=>eprintln!("Media backend not connected: {p}. Enter skips intro/video in development."),
            e=>eprintln!("Host effect pending integration: {e:?}")
        }}
    }
}
impl ApplicationHandler for Host{
    fn resumed(&mut self,ev:&ActiveEventLoop){
        if self.gpu.is_none(){
            let w=Arc::new(ev.create_window(Window::default_attributes().with_title("Black Ops Press Kit — Rust development").with_inner_size(winit::dpi::LogicalSize::new(1280.,720.))).expect("window"));
            self.gpu=Some(pollster::block_on(Gpu::new(w)));self.last=Instant::now();
        }
    }
    fn window_event(&mut self,ev:&ActiveEventLoop,_:WindowId,event:WindowEvent){
        match event{
            WindowEvent::CloseRequested=>ev.exit(),
            WindowEvent::Resized(s)=>if let Some(g)=&mut self.gpu{g.resize(s.width,s.height);},
            WindowEvent::CursorMoved{position,..}=>{self.pointer=(position.x,position.y);self.app.controls.activity();}
            WindowEvent::MouseInput{state:ElementState::Released,button:MouseButton::Left,..}=>{
                if let Some(g)=&self.gpu{let s=g.window.inner_size();if let Some((x,y))=Stage::fit(s.width.into(),s.height.into()).point(self.pointer.0,self.pointer.1){let h=self.app.hit(x,y);self.app.activate(h,x);}}
            }
            WindowEvent::KeyboardInput{event,..} if event.state==ElementState::Pressed=>{
                use winit::keyboard::{Key,NamedKey};
                match event.logical_key{
                    Key::Named(NamedKey::Enter)=>if self.app.scene()==Scene::Launcher{self.app.activate(Hit::Accept,0.);}else{self.app.media_complete();},
                    Key::Named(NamedKey::Escape)=>if matches!(self.app.scene(),Scene::Gallery|Scene::Video){self.app.activate(Hit::Close,0.);}else{ev.exit();},
                    Key::Named(NamedKey::ArrowLeft)=>self.app.activate(Hit::Previous,0.),
                    Key::Named(NamedKey::ArrowRight)=>self.app.activate(Hit::Next,0.),
                    _=>{}
                }
            }
            WindowEvent::RedrawRequested=>{
                let now=Instant::now();self.app.advance((now-self.last).as_secs_f64()*1000.);self.last=now;self.dispatch(ev);
                if let Some(g)=&mut self.gpu{g.draw(&self.root,&self.app,&self.photo);}
            }
            _=>{}
        }
        self.dispatch(ev);
    }
    fn about_to_wait(&mut self,_:&ActiveEventLoop){if let Some(g)=&self.gpu{g.window.request_redraw();}}
}
fn main(){
    let root=std::env::args_os().nth(1).map(PathBuf::from).expect("usage: presskit-app <prepared assets>");
    let event_loop=EventLoop::new().expect("event loop");
    let mut host=Host{root,app:App::default(),gpu:None,pointer:(0.,0.),last:Instant::now(),photo:None};event_loop.run_app(&mut host).expect("run");
}
