use presskit_renderer::{Renderer,foreground::{ForegroundArt,ForegroundTimeline},menu::MenuPack};
use std::path::Path;

#[test]
fn source_foreground_gpu_matches_all_105_reference_frames(){
 let Ok(assets)=std::env::var("PRESSKIT_FG_ASSETS") else{eprintln!("set PRESSKIT_FG_ASSETS to art/foreground-source and PRESSKIT_FG_REFERENCE to art/foreground");return;};
 let reference=std::env::var("PRESSKIT_FG_REFERENCE").expect("PRESSKIT_FG_REFERENCE");
 let assets=Path::new(&assets);let reference=Path::new(&reference);let bytes=std::fs::read(assets.join("timeline.pfg")).unwrap();
 let timeline=ForegroundTimeline::read(&bytes).unwrap();if matches!(bytes[3],b'3'|b'4'|b'5'|b'6'|b'7'){assert_eq!((0..timeline.count()).filter(|&i|timeline.filename(i).ends_with(".jpg")).count(),26);}else{assert_eq!(timeline.count(),49);}
 pollster::block_on(async{
  let instance=wgpu::Instance::default();let adapter=instance.request_adapter(&Default::default()).await.unwrap();eprintln!("foreground GPU adapter: {:?}",adapter.get_info());
  let (d,q)=adapter.request_device(&wgpu::DeviceDescriptor::default(),None).await.unwrap();
  let r=Renderer::new(&d,&q,wgpu::TextureFormat::Rgba8UnormSrgb,include_str!("../../../shaders/generated/vertex.wgsl"),include_str!("../../../shaders/generated/fragment.wgsl"),"vertexMain","fragmentMain");
  let mut art=ForegroundArt::new(timeline);
  if art.needs_menu(){art.install_menu(&std::fs::read(assets.join("menu.pkm")).unwrap()).unwrap();}
  for i in art.missing(){let im=image::open(assets.join(art.filename(i))).unwrap().into_rgba8();art.install(i,r.layer(&d,&q,im.as_raw(),im.width(),im.height(),[0.,0.,1.,1.])).unwrap();}
  let size=wgpu::Extent3d{width:1280,height:720,depth_or_array_layers:1};
  let target=d.create_texture(&wgpu::TextureDescriptor{label:Some("foreground reference target"),size,mip_level_count:1,sample_count:1,dimension:wgpu::TextureDimension::D2,format:wgpu::TextureFormat::Rgba8UnormSrgb,usage:wgpu::TextureUsages::RENDER_ATTACHMENT|wgpu::TextureUsages::COPY_SRC,view_formats:&[]});let view=target.create_view(&Default::default());
  let buffer=d.create_buffer(&wgpu::BufferDescriptor{label:Some("foreground readback"),size:1280*720*4,usage:wgpu::BufferUsages::COPY_DST|wgpu::BufferUsages::MAP_READ,mapped_at_creation:false});
  let draw=|layers:&[&presskit_renderer::Layer],filters:&[presskit_renderer::FilterSpan]|->Vec<u8>{
   let mut enc=d.create_command_encoder(&Default::default());r.render_filtered(&mut enc,&view,layers,filters,[0.,0.,1280.,720.]);
   enc.copy_texture_to_buffer(wgpu::TexelCopyTextureInfo{texture:&target,mip_level:0,origin:wgpu::Origin3d::ZERO,aspect:wgpu::TextureAspect::All},wgpu::TexelCopyBufferInfo{buffer:&buffer,layout:wgpu::TexelCopyBufferLayout{offset:0,bytes_per_row:Some(5120),rows_per_image:Some(720)}},size);q.submit([enc.finish()]);
   let slice=buffer.slice(..);let(tx,rx)=std::sync::mpsc::channel();slice.map_async(wgpu::MapMode::Read,move|v|tx.send(v).unwrap());d.poll(wgpu::Maintain::Wait);rx.recv().unwrap().unwrap();let data=slice.get_mapped_range();let out=data.to_vec();drop(data);buffer.unmap();out
  };
  let mut failures=Vec::new();
  let diagnostic=std::env::var("PRESSKIT_GPU_DIAGNOSTICS").ok().map(std::path::PathBuf::from);
  if let Some(p)=&diagnostic{std::fs::create_dir_all(p).unwrap();}
  if let Ok(root)=std::env::var("PRESSKIT_LAUNCHER_ASSETS"){
   let bytes=std::fs::read(Path::new(&root).join("vector/launcher.pkv")).unwrap();
   assert!(bytes.starts_with(b"PKL2"));
   let mut launcher=presskit_renderer::launcher::LauncherArt::read(&bytes,&r,&d,&q).unwrap();
   let normal=draw(&launcher.layers().collect::<Vec<_>>(),&[]);
   let reference=image::open(Path::new(&root).join("launcher/1.png")).unwrap().into_rgba8();
   assert_eq!(reference.dimensions(),(500,300));
   let mut sum=0f64;
   for y in 0..300usize{for x in 0..500usize{
    let got=&normal[((210+y)*1280+390+x)*4..];let e=reference.get_pixel(x as u32,y as u32).0;
    for c in 0..3{sum+=(f64::from(got[c])-f64::from(e[c])*f64::from(e[3])/255.).abs();}
   }}
   let mae=sum/(500.*300.*3.);eprintln!("native launcher MAE {mae:.4}/255");if mae>=1.5{failures.push(format!("launcher MAE {mae:.4} >= 1.5"));}
   let mut rendered=image::RgbaImage::new(500,300);let mut difference=image::RgbaImage::new(500,300);
   for y in 0..300usize{for x in 0..500usize{
    let g=&normal[((210+y)*1280+390+x)*4..];let e=reference.get_pixel(x as u32,y as u32).0;
    rendered.put_pixel(x as u32,y as u32,image::Rgba([g[0],g[1],g[2],255]));
    let err=|c:usize|((f64::from(g[c])-f64::from(e[c])*f64::from(e[3])/255.).abs()*4.).min(255.) as u8;
    difference.put_pixel(x as u32,y as u32,image::Rgba([err(0),err(1),err(2),255]));
   }}
   if let Some(p)=&diagnostic{rendered.save(p.join("launcher-gpu.png")).unwrap();reference.save(p.join("launcher-original.png")).unwrap();difference.save(p.join("launcher-error-x4.png")).unwrap();}
   for y in (0..300).step_by(50){let mut row=Vec::new();for x in (0..500).step_by(50){
    let mut error=0f64;let mut signed=0f64;
    for yy in y..y+50{for xx in x..x+50{let g=rendered.get_pixel(xx,yy).0;let e=reference.get_pixel(xx,yy).0;for c in 0..3{let d=f64::from(g[c])-f64::from(e[c])*f64::from(e[3])/255.;error+=d.abs();signed+=d;}}}
    row.push(format!("{:.2}/{:+.2}",error/7500.,signed/7500.));
   }eprintln!("launcher y={y} tile MAE/bias: {}",row.join(" "));}
   for selected in [1,2]{
    launcher.update(&r,&q,selected,false);let hover=draw(&launcher.layers().collect::<Vec<_>>(),&[]);
    launcher.update(&r,&q,selected,true);let down=draw(&launcher.layers().collect::<Vec<_>>(),&[]);
    if hover==normal{failures.push(format!("launcher button {selected} hover unchanged"));}if down!=hover{failures.push(format!("launcher button {selected} shared hover/down differs"));}
   }
   launcher.update(&r,&q,0,false);assert_eq!(draw(&launcher.layers().collect::<Vec<_>>(),&[]),normal);
  }
  // Group alpha must be applied once, including when two opaque meshes overlap.
  let a=r.layer(&d,&q,&[255,0,0,255],1,1,[100.,100.,100.,100.]);
  let b=r.layer(&d,&q,&[255,0,0,255],1,1,[150.,100.,100.,100.]);
  let span=presskit_renderer::FilterSpan{start:0,end:2,alpha:0.5,blur_y:0.};
  let pixels=draw(&[&a,&b],&[span]);
  let pixel=|x:usize,y:usize|pixels[(y*1280+x)*4];
  if pixel(125,125).abs_diff(128)>1||pixel(175,125).abs_diff(128)>1{failures.push(format!("group alpha samples {}, {}",pixel(125,125),pixel(175,125)));}
  let blurred=draw(&[&a,&b],&[presskit_renderer::FilterSpan{blur_y:15.,..span}]);
  if blurred[(95*1280+175)*4]==0{failures.push("blur did not extend above contour".into());}
  let mut worst=(0u16,0.0f64);
  for frame in 1..=105u16{
   art.update(&r,&q,frame);art.update_menu(&r,&d,&q,0,false);let (layers,filters)=art.layers_and_filters();let got=draw(&layers,&filters);
   let im=image::open(reference.join(format!("{frame}.png"))).unwrap().into_rgba8();assert_eq!(im.dimensions(),(1280,720));
   let sum:f64=got.chunks_exact(4).zip(im.as_raw().chunks_exact(4)).map(|(g,e)|(0..3).map(|c|(f64::from(g[c])-f64::from(e[c])*f64::from(e[3])/255.).abs()).sum::<f64>()).sum();
   let mae=sum/(1280.*720.*3.);if mae>worst.1{worst=(frame,mae);}eprintln!("foreground frame {frame:03} MAE {mae:.4}/255");
   if mae>=1.5{if let Some(p)=&diagnostic{image::save_buffer(p.join(format!("frame-{frame:03}-gpu.png")),&got,1280,720,image::ColorType::Rgba8).unwrap();im.save(p.join(format!("frame-{frame:03}-original.png"))).unwrap();}}
  }
  eprintln!("worst foreground GPU gamma MAE: frame {} {:.4}/255",worst.0,worst.1);if worst.1>=1.5{failures.push(format!("foreground frame {} MAE {:.4} >= 1.5",worst.0,worst.1));}
  if matches!(bytes[3],b'2'|b'3'|b'4'|b'5'|b'6'|b'7'){
   let menu_bytes=std::fs::read(assets.join("menu.pkm")).unwrap();let pack=MenuPack::read(&menu_bytes).unwrap();
   if menu_bytes.starts_with(b"PKM3"){
    for frame in 1..=105u16{assert_eq!(pack.group(frame)==3,[77,79,81,82,83,84].contains(&frame));}
    assert!(!pack.label_layers(&r,&d,&q,3).is_empty(),"wallpaper glyph geometry missing");
    assert!(pack.layers(&r,&d,&q,3,1,true).is_empty(),"wallpaper label group must not draw trailer buttons");
   }
   let white=r.layer(&d,&q,&[255,255,255,255],1,1,[0.,0.,1280.,720.]);
   for (group,y,normal,down) in [(1u8,240usize,[153u8;3],[102u8;3]),(2,190,[167,165,166],[123,121,121])]{
    for (selected,pressed,expected) in [(0,false,normal),(1,false,[255;3]),(1,true,down)]{
     let geometry=pack.layers(&r,&d,&q,group,selected,pressed);assert!(geometry.len()<=4);let mut layers=vec![&white];layers.extend(geometry.iter());let data=draw(&layers,&[]);
     let sample=|x:usize|&data[(y*1280+x)*4..(y*1280+x)*4+3];
     for (got,exp) in sample(565).iter().zip(expected){assert!(got.abs_diff(exp)<=1,"group {group}, selected {selected}, pressed {pressed}");}
     for (got,exp) in sample(if group==1{750}else{675}).iter().zip(normal){assert!(got.abs_diff(exp)<=1,"non-selected button changed");}
    }
   }
   eprintln!("All six menu states and non-selected instances passed GPU color checks");
  }
  assert!(failures.is_empty(),"{}",failures.join("; "));
 });
}
