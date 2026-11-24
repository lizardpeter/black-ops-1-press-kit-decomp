mod media;
use media::Media;
use presskit_core::{App,Effect,Hit,Scene,Stage};
use presskit_renderer::{Renderer,Layer};
use presskit_renderer::background::{BackgroundArt,BackgroundTimeline};
use presskit_renderer::foreground::{ForegroundArt,ForegroundTimeline};
use presskit_renderer::control_pack::{ControlPack,ControlCache,ControlVisual};
use std::{sync::Arc,path::PathBuf,time::Instant};
use winit::{application::ApplicationHandler,event::{WindowEvent,ElementState,MouseButton},event_loop::{ActiveEventLoop,EventLoop},window::{Window,WindowId}};
const VERTEX:&str=include_str!("../../../shaders/generated/vertex.wgsl");
const FRAGMENT:&str=include_str!("../../../shaders/generated/fragment.wgsl");
struct Gpu{
    window:Arc<Window>,surface:wgpu::Surface<'static>,device:wgpu::Device,queue:wgpu::Queue,config:wgpu::SurfaceConfiguration,renderer:Renderer,
    art:Option<Layer>,background:Option<BackgroundArt>,foreground:Option<ForegroundArt>,last_art:String,video:Option<Layer>,last_video:u64,chrome:Option<Layer>,chrome_kind:Scene,pack:Option<ControlPack>,launcher:Vec<Layer>,controls:ControlCache
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
        let renderer=Renderer::new(&device,&queue,config.format,VERTEX,FRAGMENT,"vertexMain","fragmentMain");
        Self{window,surface,device,queue,config,renderer,art:None,background:None,foreground:None,last_art:String::new(),video:None,last_video:0,chrome:None,chrome_kind:Scene::Launcher,pack:None,launcher:vec![],controls:ControlCache::default()}
    }
    fn resize(&mut self,w:u32,h:u32){if w>0&&h>0{self.config.width=w;self.config.height=h;self.surface.configure(&self.device,&self.config);}}
    fn image(&self,path:&std::path::Path,rect:[f32;4])->Option<Layer>{
        let img=image::open(path).map_err(|e|eprintln!("{}: {e}",path.display())).ok()?.into_rgba8();
        Some(self.renderer.layer(&self.device,&self.queue,img.as_raw(),img.width(),img.height(),rect))
    }
    fn draw(&mut self,root:&std::path::Path,app:&App,photo:&Option<String>,media:&Media,states:[u8;4],hit:Hit,pressed:bool){
        let layered=(self.foreground.is_some()||root.join("art/foreground/75.png").is_file())&&self.background.as_ref().is_some_and(|b|b.ready());
        let fg_on=layered&&self.foreground.is_some()&&matches!(app.scene(),Scene::Transition|Scene::Welcome|Scene::Wallpapers|Scene::Videos|Scene::Screens);
        let (path,rect)=match app.scene(){
            Scene::Launcher=>("art/launcher/1.png".to_string(),[390.,210.,500.,300.]),
            Scene::Gallery=>(photo.clone().unwrap_or_default(),[0.,0.,1280.,720.]),
            Scene::Intro|Scene::Video=>(String::new(),[0.,0.,1280.,720.]),
            _ if fg_on=>(String::new(),[0.,0.,1280.,720.]),
            _=>(format!("art/{}/{}.png",if layered{"foreground"}else{"presskit"},app.frame),[0.,0.,1280.,720.])
        };
        if path!=self.last_art{self.art=if path.is_empty(){None}else{self.image(&root.join(&path),rect)};self.last_art=path;}
        let bg_on=layered&&app.frame>=56&&!matches!(app.scene(),Scene::Gallery|Scene::Video);
        if bg_on{if let Some(bg)=&mut self.background{bg.update(&self.renderer,&self.device,&self.queue,app.background.frame);}}
        if fg_on{self.foreground.as_mut().unwrap().update(&self.renderer,&self.queue,app.frame);}
        if fg_on{let selected=match hit{Hit::Trailer(id)|Hit::Thumbnail(id)|Hit::Wallpaper(id)=>id,Hit::Quit=>255,Hit::Website=>240,Hit::Screens=>241,Hit::Videos=>242,Hit::Wallpapers=>243,_=>0};self.foreground.as_mut().unwrap().update_menu(&self.renderer,&self.device,&self.queue,selected,pressed);}
        let media_on=matches!(app.scene(),Scene::Video|Scene::Intro);
        if media_on&&self.last_video!=media.revision{self.last_video=media.revision;if let Some(p)=&media.frame{
            if let Some(v)=&self.video{self.queue.write_texture(wgpu::TexelCopyTextureInfo{texture:&v.texture,mip_level:0,origin:wgpu::Origin3d::ZERO,aspect:wgpu::TextureAspect::All},p,wgpu::TexelCopyBufferLayout{offset:0,bytes_per_row:Some(media::WIDTH*4),rows_per_image:Some(media::HEIGHT)},wgpu::Extent3d{width:media::WIDTH,height:media::HEIGHT,depth_or_array_layers:1});}else{self.video=Some(self.renderer.layer(&self.device,&self.queue,p,media::WIDTH,media::HEIGHT,[0.,0.,1280.,720.]));}
        }else{self.video=None;}}
        if self.pack.is_none()&&matches!(app.scene(),Scene::Video|Scene::Gallery){if self.chrome_kind!=app.scene(){self.chrome=self.image(&root.join(if app.scene()==Scene::Video{"art/video_controls/1.png"}else{"art/gallery_controls/1.png"}),[200.,app.controls.y as f32,880.,90.]);self.chrome_kind=app.scene();}if let Some(c)=&mut self.chrome{self.renderer.reposition(&self.device,c,[200.,app.controls.y as f32,880.,90.]);}}else{self.chrome=None;self.chrome_kind=Scene::Launcher;}
        if matches!(app.scene(),Scene::Video|Scene::Gallery){
            let gallery=app.scene()==Scene::Gallery;let text=if gallery{app.gallery.counter()}else{format!("{}/{}",presskit_core::flash_time(media.position()),presskit_core::flash_time(media.duration()))};
            self.controls.update(self.pack.as_ref(),&self.renderer,&self.device,&self.queue,ControlVisual{y:app.controls.y as f32,gallery,paused:media.paused,progress:if media.duration()>0.{(media.position()/media.duration())as f32}else{0.},text,states});
        }else{self.controls.clear();}
        let size=self.window.inner_size();if size.width==0||size.height==0{return;}
        let frame=match self.surface.get_current_texture(){Ok(f)=>f,Err(wgpu::SurfaceError::Lost|wgpu::SurfaceError::Outdated)=>{self.resize(size.width,size.height);return;},Err(e)=>{eprintln!("surface: {e}");return;}};
        let view=frame.texture.create_view(&Default::default());
        let mut encoder=self.device.create_command_encoder(&Default::default());
        let stage=Stage::fit(f64::from(size.width),f64::from(size.height));
        let mut layers=vec![];if bg_on{if let Some(b)=&self.background{layers.extend(b.layers(app.background.frame));}}if fg_on{layers.extend(self.foreground.as_ref().unwrap().layers());}if let Some(a)=&self.art{layers.push(a);if app.scene()==Scene::Launcher{layers.extend(self.launcher.iter());}}if media_on{if let Some(v)=&self.video{layers.push(v);}}if let Some(c)=&self.chrome{layers.push(c);}layers.extend(self.controls.layers().iter());
        self.renderer.render(&mut encoder,&view,&layers,[stage.x as f32,stage.y as f32,(1280.*stage.scale) as f32,(720.*stage.scale) as f32]);
        self.queue.submit([encoder.finish()]);frame.present();
    }
}
struct Host{root:PathBuf,app:App,gpu:Option<Gpu>,pointer:(f64,f64),last:Instant,photo:Option<String>,media:Media,pressed:bool}
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
            Effect::Play(p)=>if let Err(e)=self.media.play(&self.root.join(p)){eprintln!("Media {p}: {e}");if let Some(g)=&self.gpu{g.window.set_title(&format!("Media error: {e} — Enter to skip"));}},
            Effect::StopVideo=>self.media.stop(),
            Effect::ToggleVideo=>{if let Err(e)=self.media.toggle(){eprintln!("pause: {e}");}},
            Effect::Seek(v)=>{if let Err(e)=self.media.seek(v){eprintln!("seek: {e}");}},
            Effect::Music(on)=>self.media.music(&self.root.join("art/sounds/1.mp3"),on),
            Effect::Sound(id)=>self.media.sound(&self.root.join(format!("art/sounds/{id}.mp3"))),
            e=>eprintln!("Host effect pending integration: {e:?}")
        }}
    }
}
impl ApplicationHandler for Host{
    fn resumed(&mut self,ev:&ActiveEventLoop){
        if self.gpu.is_none(){
            let w=Arc::new(ev.create_window(Window::default_attributes().with_title("Black Ops Press Kit — Rust development").with_inner_size(winit::dpi::LogicalSize::new(1280.,720.))).expect("window"));
            let mut gpu=pollster::block_on(Gpu::new(w));if let Ok(bytes)=std::fs::read(self.root.join("art/vector/launcher.pkv")){if let Ok(p)=ControlPack::read_launcher(&bytes){gpu.launcher=p.launcher_layers(&gpu.renderer,&gpu.device,&gpu.queue);}}gpu.pack=std::fs::read(self.root.join("art/vector/controls.pkv")).ok().and_then(|b|ControlPack::read(&b).ok());if let Ok(bytes)=std::fs::read(self.root.join("art/background-source/timeline.pbg")){if let Ok(t)=BackgroundTimeline::read(&bytes){let mut bg=BackgroundArt::new(t);for i in 0..4{if let Some(l)=gpu.image(&self.root.join(format!("art/background-source/{i}.jpg")),[0.,0.,1280.,720.]){let _=bg.install(i,l);}}if bg.ready(){gpu.background=Some(bg);}}}if let Ok(bytes)=std::fs::read(self.root.join("art/foreground-source/timeline.pfg")){if let Ok(t)=ForegroundTimeline::read(&bytes){let mut fg=ForegroundArt::new(t);if fg.needs_menu(){if let Ok(b)=std::fs::read(self.root.join("art/foreground-source/menu.pkm")){let _=fg.install_menu(&b);}}for i in fg.missing(){if let Some(l)=gpu.image(&self.root.join(format!("art/foreground-source/{}",fg.filename(i))),[0.,0.,1.,1.]){let _=fg.install(i,l);}}if fg.missing().is_empty()&&!fg.needs_menu(){gpu.foreground=Some(fg);}}}self.gpu=Some(gpu);self.last=Instant::now();
        }
    }
    fn window_event(&mut self,ev:&ActiveEventLoop,_:WindowId,event:WindowEvent){
        match event{
            WindowEvent::CloseRequested=>ev.exit(),
            WindowEvent::Resized(s)=>if let Some(g)=&mut self.gpu{g.resize(s.width,s.height);},
            WindowEvent::CursorMoved{position,..}=>{self.pointer=(position.x,position.y);self.app.controls.activity();}
            WindowEvent::MouseInput{state:ElementState::Pressed,button:MouseButton::Left,..}=>{self.pressed=true;}
            WindowEvent::MouseInput{state:ElementState::Released,button:MouseButton::Left,..}=>{self.pressed=false;
                if let Some(g)=&self.gpu{let s=g.window.inner_size();if let Some((x,y))=Stage::fit(s.width.into(),s.height.into()).point(self.pointer.0,self.pointer.1){let h=self.app.hit(x,y);self.app.activate(h,x);}}
            }
            WindowEvent::KeyboardInput{event,..} if event.state==ElementState::Pressed=>{
                use winit::keyboard::{Key,NamedKey};
                match event.logical_key{
                    Key::Named(NamedKey::Enter)=>if self.app.scene()==Scene::Launcher{self.app.activate(Hit::Accept,0.);}else{self.media.stop();self.app.media_complete();},
                    Key::Named(NamedKey::Escape)=>if matches!(self.app.scene(),Scene::Gallery|Scene::Video){self.app.activate(Hit::Close,0.);}else{ev.exit();},
                    Key::Named(NamedKey::Space)=>self.app.activate(Hit::PlayPause,0.),
                    Key::Named(NamedKey::ArrowLeft)=>self.app.activate(Hit::Previous,0.),
                    Key::Named(NamedKey::ArrowRight)=>self.app.activate(Hit::Next,0.),
                    _=>{}
                }
            }
            WindowEvent::RedrawRequested=>{
                let now=Instant::now();self.app.advance((now-self.last).as_secs_f64()*1000.);self.last=now;if self.media.update(){self.app.media_complete();self.media.stop();}self.dispatch(ev);
                if let Some(g)=&mut self.gpu{let size=g.window.inner_size();let hit=Stage::fit(size.width.into(),size.height.into()).point(self.pointer.0,self.pointer.1).map(|(x,y)|self.app.hit(x,y)).unwrap_or(Hit::None);g.draw(&self.root,&self.app,&self.photo,&self.media,presskit_core::control_states(hit,self.pressed),hit,self.pressed);}
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
    let mut host=Host{root,app:App::default(),gpu:None,pointer:(0.,0.),last:Instant::now(),photo:None,media:Media::default(),pressed:false};event_loop.run_app(&mut host).expect("run");
}
