use presskit_renderer::{Renderer,foreground::{ForegroundArt,ForegroundTimeline},menu::MenuPack};
use std::path::Path;

#[test]
fn source_foreground_gpu_matches_all_105_reference_frames(){
 let Ok(assets)=std::env::var("PRESSKIT_FG_ASSETS") else{eprintln!("set PRESSKIT_FG_ASSETS to art/foreground-source and PRESSKIT_FG_REFERENCE to art/foreground");return;};
 let reference=std::env::var("PRESSKIT_FG_REFERENCE").expect("PRESSKIT_FG_REFERENCE");
 let assets=Path::new(&assets);let reference=Path::new(&reference);let bytes=std::fs::read(assets.join("timeline.pfg")).unwrap();
 let timeline=ForegroundTimeline::read(&bytes).unwrap();if matches!(bytes[3],b'3'|b'4'|b'5'|b'6'){assert_eq!((0..timeline.count()).filter(|&i|timeline.filename(i).ends_with(".jpg")).count(),26);}else{assert_eq!(timeline.count(),49);}
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
  let draw=|layers:&[&presskit_renderer::Layer]|->Vec<u8>{
   let mut enc=d.create_command_encoder(&Default::default());r.render(&mut enc,&view,layers,[0.,0.,1280.,720.]);
   enc.copy_texture_to_buffer(wgpu::TexelCopyTextureInfo{texture:&target,mip_level:0,origin:wgpu::Origin3d::ZERO,aspect:wgpu::TextureAspect::All},wgpu::TexelCopyBufferInfo{buffer:&buffer,layout:wgpu::TexelCopyBufferLayout{offset:0,bytes_per_row:Some(5120),rows_per_image:Some(720)}},size);q.submit([enc.finish()]);
   let slice=buffer.slice(..);let(tx,rx)=std::sync::mpsc::channel();slice.map_async(wgpu::MapMode::Read,move|v|tx.send(v).unwrap());d.poll(wgpu::Maintain::Wait);rx.recv().unwrap().unwrap();let data=slice.get_mapped_range();let out=data.to_vec();drop(data);buffer.unmap();out
  };
  let mut worst=(0u16,0.0f64);
  for frame in 1..=105u16{
   art.update(&r,&q,frame);art.update_menu(&r,&d,&q,0,false);let got=draw(&art.layers());
   let im=image::open(reference.join(format!("{frame}.png"))).unwrap().into_rgba8();assert_eq!(im.dimensions(),(1280,720));
   let sum:f64=got.chunks_exact(4).zip(im.as_raw().chunks_exact(4)).map(|(g,e)|(0..3).map(|c|(f64::from(g[c])-f64::from(e[c])*f64::from(e[3])/255.).abs()).sum::<f64>()).sum();
   let mae=sum/(1280.*720.*3.);if mae>worst.1{worst=(frame,mae);}
  }
  eprintln!("worst foreground GPU gamma MAE: frame {} {:.4}/255",worst.0,worst.1);assert!(worst.1<1.5);
  if matches!(bytes[3],b'2'|b'3'|b'4'|b'5'|b'6'){
   let menu_bytes=std::fs::read(assets.join("menu.pkm")).unwrap();let pack=MenuPack::read(&menu_bytes).unwrap();
   if menu_bytes.starts_with(b"PKM3"){
    for frame in 1..=105u16{assert_eq!(pack.group(frame)==3,[77,79,81,82,83,84].contains(&frame));}
    assert!(!pack.label_layers(&r,&d,&q,3).is_empty(),"wallpaper glyph geometry missing");
    assert!(pack.layers(&r,&d,&q,3,1,true).is_empty(),"wallpaper label group must not draw trailer buttons");
   }
   let white=r.layer(&d,&q,&[255,255,255,255],1,1,[0.,0.,1280.,720.]);
   for (group,y,normal,down) in [(1u8,240usize,[153u8;3],[102u8;3]),(2,190,[167,165,166],[123,121,121])]{
    for (selected,pressed,expected) in [(0,false,normal),(1,false,[255;3]),(1,true,down)]{
     let geometry=pack.layers(&r,&d,&q,group,selected,pressed);assert!(geometry.len()<=4);let mut layers=vec![&white];layers.extend(geometry.iter());let data=draw(&layers);
     let sample=|x:usize|&data[(y*1280+x)*4..(y*1280+x)*4+3];
     for (got,exp) in sample(565).iter().zip(expected){assert!(got.abs_diff(exp)<=1,"group {group}, selected {selected}, pressed {pressed}");}
     for (got,exp) in sample(if group==1{750}else{675}).iter().zip(normal){assert!(got.abs_diff(exp)<=1,"non-selected button changed");}
    }
   }
   eprintln!("All six menu states and non-selected instances passed GPU color checks");
  }
 });
}
