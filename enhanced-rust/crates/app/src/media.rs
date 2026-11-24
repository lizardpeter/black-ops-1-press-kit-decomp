//! Interim desktop media backend: external FFmpeg decodes RGBA, FFplay outputs audio.
//! Keep decoding off the UI thread and bound queued frames. Browser uses its own native media API.
use std::{io::{self,Read},path::{Path,PathBuf},process::{Child,Command,Stdio},sync::mpsc::{self,Receiver,TryRecvError},thread,time::Instant};
pub const WIDTH:u32=1280;pub const HEIGHT:u32=720;const FPS:f64=30.;
struct Stream{child:Child,rx:Receiver<Vec<u8>>,audio:Option<Child>,started:Instant,frames:u64}
impl Drop for Stream{fn drop(&mut self){let _=self.child.kill();let _=self.child.wait();if let Some(c)=&mut self.audio{let _=c.kill();let _=c.wait();}}}
#[derive(Default)]pub struct Media{file:Option<PathBuf>,stream:Option<Stream>,offset:f64,duration:f64,pub frame:Option<Vec<u8>>,pub paused:bool,pub revision:u64,music:Option<Child>,effects:Vec<Child>}
fn probe(path:&Path)->io::Result<f64>{let o=Command::new("ffprobe").args(["-v","error","-show_entries","format=duration","-of","default=noprint_wrappers=1:nokey=1"]).arg(path).output()?;if !o.status.success(){return Err(io::Error::other("ffprobe rejected media"));}let d=String::from_utf8_lossy(&o.stdout).trim().parse::<f64>().map_err(io::Error::other)?;if d.is_finite()&&d>0.{Ok(d)}else{Err(io::Error::other("invalid duration"))}}
fn audio(path:&Path,offset:f64,looped:bool)->io::Result<Child>{let mut c=Command::new("ffplay");c.args(["-hide_banner","-loglevel","error","-nodisp","-autoexit","-vn","-ss",&format!("{offset:.4}")]);if looped{c.args(["-loop","0"]);}c.arg(path).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn()}
fn kill(c:&mut Option<Child>){if let Some(mut p)=c.take(){let _=p.kill();let _=p.wait();}}
impl Media{
    pub fn play(&mut self,path:&Path)->io::Result<()>{self.stop();self.duration=probe(path)?;self.file=Some(path.to_owned());self.offset=0.;self.resume()}
    fn resume(&mut self)->io::Result<()>{let path=self.file.as_ref().ok_or_else(||io::Error::other("no media"))?;let mut child=Command::new("ffmpeg").args(["-nostdin","-hide_banner","-loglevel","error","-ss",&format!("{:.4}",self.offset),"-i"]).arg(path).args(["-an","-vf","fps=30,scale=1280:720","-pix_fmt","rgba","-f","rawvideo","pipe:1"]).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn()?;let mut stdout=child.stdout.take().ok_or_else(||io::Error::other("missing decode pipe"))?;let (tx,rx)=mpsc::sync_channel(3);thread::spawn(move||loop{let mut pixels=vec![0;WIDTH as usize*HEIGHT as usize*4];if stdout.read_exact(&mut pixels).is_err()||tx.send(pixels).is_err(){break;}});self.stream=Some(Stream{child,rx,audio:None,started:Instant::now(),frames:0});self.paused=false;Ok(())}
    pub fn stop(&mut self){self.stream.take();self.file=None;self.frame=None;self.offset=0.;self.duration=0.;self.paused=false;self.revision+=1;}
    pub fn position(&self)->f64{if let Some(s)=&self.stream{(self.offset+s.frames.saturating_sub(1) as f64/FPS).min(self.duration)}else{self.offset}}
    pub fn duration(&self)->f64{self.duration}
    pub fn toggle(&mut self)->io::Result<()>{if self.file.is_none(){return Ok(());}if self.paused{self.resume()}else{self.offset=self.position();self.stream.take();self.paused=true;Ok(())}}
    pub fn seek(&mut self,fraction:f64)->io::Result<()>{if self.file.is_none(){return Ok(());}let was_paused=self.paused;self.stream.take();self.offset=(fraction.clamp(0.,1.)*self.duration).min((self.duration-1./FPS).max(0.));self.resume()?;if was_paused{self.paused=true;}Ok(())}
    /// True on decoder EOF; video clock advances according to decoded 30 Hz frames.
    pub fn update(&mut self)->bool{
        let Some(s)=&mut self.stream else{return false;};
        let desired=if s.frames==0{1}else{(s.started.elapsed().as_secs_f64()*FPS).floor() as u64+1};
        if self.paused&&s.frames>0{return false;}
        for _ in 0..3{if s.frames>=desired{break;}match s.rx.try_recv(){
            Ok(pixels)=>{if s.frames==0{s.started=Instant::now();if !self.paused{if let Some(path)=&self.file{s.audio=audio(path,self.offset,false).ok();}}}s.frames+=1;self.frame=Some(pixels);self.revision+=1;},
            Err(TryRecvError::Empty)=>break,Err(TryRecvError::Disconnected)=>{self.stream.take();return true;}
        }}false
    }
    pub fn music(&mut self,path:&Path,on:bool){if on{if self.music.is_none(){self.music=audio(path,0.,true).ok();}}else{kill(&mut self.music);}}
    pub fn sound(&mut self,path:&Path){self.effects.retain_mut(|c|c.try_wait().ok().flatten().is_none());if self.effects.len()>=4{let mut c=self.effects.remove(0);let _=c.kill();let _=c.wait();}if let Ok(c)=audio(path,0.,false){self.effects.push(c);}}
}
impl Drop for Media{fn drop(&mut self){self.stop();kill(&mut self.music);for c in &mut self.effects{let _=c.kill();let _=c.wait();}}}
