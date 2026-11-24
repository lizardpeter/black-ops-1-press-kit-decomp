use presskit_renderer::{Renderer,background::{BackgroundArt,BackgroundTimeline}};
#[test]
fn original_crossfade_gpu(){
 let Ok(root)=std::env::var("PRESSKIT_BG_ASSETS") else{eprintln!("set PRESSKIT_BG_ASSETS to prepared art directory");return;};let root=std::path::Path::new(&root);
 let bytes=std::fs::read(root.join("background-source/timeline.pbg")).unwrap();assert_eq!(bytes.len(),1130);let timeline=BackgroundTimeline::read(&bytes).unwrap();
 assert_eq!(timeline.frame(1)[0].alpha,241);assert_eq!(timeline.frame(40).len(),2);assert_eq!(timeline.frame(221)[1].image,0);
 pollster::block_on(async{
  let i=wgpu::Instance::default();let a=i.request_adapter(&Default::default()).await.unwrap();let (d,q)=a.request_device(&Default::default(),None).await.unwrap();let r=Renderer::new(&d,&q,wgpu::TextureFormat::Rgba8UnormSrgb,include_str!("../../../shaders/generated/vertex.wgsl"),include_str!("../../../shaders/generated/fragment.wgsl"),"vertexMain","fragmentMain");
  let mut bg=BackgroundArt::new(timeline);for i in 0..4{let im=image::open(root.join(format!("background-source/{i}.jpg"))).unwrap().into_rgba8();bg.install(i,r.layer(&d,&q,im.as_raw(),im.width(),im.height(),[0.,0.,1280.,720.])).unwrap();}assert!(bg.ready());
  let size=wgpu::Extent3d{width:1280,height:720,depth_or_array_layers:1};let target=d.create_texture(&wgpu::TextureDescriptor{label:None,size,mip_level_count:1,sample_count:1,dimension:wgpu::TextureDimension::D2,format:wgpu::TextureFormat::Rgba8UnormSrgb,usage:wgpu::TextureUsages::RENDER_ATTACHMENT|wgpu::TextureUsages::COPY_SRC,view_formats:&[]});let view=target.create_view(&Default::default());
  let buf=d.create_buffer(&wgpu::BufferDescriptor{label:None,size:1280*720*4,usage:wgpu::BufferUsages::MAP_READ|wgpu::BufferUsages::COPY_DST,mapped_at_creation:false});
  for f in [1,35,40,45,56,57,110,165,221]{bg.update(&r,&d,&q,f);let mut enc=d.create_command_encoder(&Default::default());r.render(&mut enc,&view,&bg.layers(f),[0.,0.,1280.,720.]);enc.copy_texture_to_buffer(wgpu::TexelCopyTextureInfo{texture:&target,mip_level:0,origin:wgpu::Origin3d::ZERO,aspect:wgpu::TextureAspect::All},wgpu::TexelCopyBufferInfo{buffer:&buf,layout:wgpu::TexelCopyBufferLayout{offset:0,bytes_per_row:Some(5120),rows_per_image:Some(720)}},size);q.submit([enc.finish()]);let slice=buf.slice(..);let(tx,rx)=std::sync::mpsc::channel();slice.map_async(wgpu::MapMode::Read,move|x|tx.send(x).unwrap());d.poll(wgpu::Maintain::Wait);rx.recv().unwrap().unwrap();let got=slice.get_mapped_range();let reference=image::open(root.join(format!("background/{f}.png"))).unwrap().into_rgba8();let sum:f64=got.chunks_exact(4).zip(reference.as_raw().chunks_exact(4)).map(|(a,b)|(0..3).map(|c|(f64::from(a[c])-f64::from(b[c])*f64::from(b[3])/255.).abs()).sum::<f64>()).sum();let mae=sum/(1280.*720.*3.);eprintln!("frame {f} MAE {mae:.3} / 255");assert!(mae<2.0,"frame {f}: {mae}");drop(got);buf.unmap();}
 });
}

#[test]
fn delayed_background_texture_keeps_last_complete_pair(){
 pollster::block_on(async{
  let instance=wgpu::Instance::default();
  let Some(adapter)=instance.request_adapter(&Default::default()).await else{eprintln!("no adapter; skipped delayed-background GPU check");return;};
  let Ok((device,queue))=adapter.request_device(&Default::default(),None).await else{eprintln!("no device; skipped delayed-background GPU check");return;};
  let renderer=Renderer::new(&device,&queue,wgpu::TextureFormat::Rgba8UnormSrgb,include_str!("../../../shaders/generated/vertex.wgsl"),include_str!("../../../shaders/generated/fragment.wgsl"),"vertexMain","fragmentMain");
  let mut bytes=b"PBG1\xdd\0".to_vec();
  for frame in 0..221{let image=if frame==0{0}else{1};bytes.extend([1,image,0,1]);}
  let timeline=BackgroundTimeline::read(&bytes).unwrap();
  let mut bg=BackgroundArt::new(timeline);
  let white=vec![255u8;1280*720*4];
  let grey=vec![64u8;1280*720*4];
  bg.install(0,renderer.layer(&device,&queue,&white,1280,720,[0.,0.,1280.,720.])).unwrap();
  assert!(bg.ready_frame(1));
  assert!(!bg.ready_frame(2));
  bg.update(&renderer,&device,&queue,1);
  assert_eq!(bg.layers(1).len(),1);
  bg.update(&renderer,&device,&queue,2);
  assert_eq!(bg.layers(2).len(),1,"missing next texture must retain the previous complete background");
  bg.install(1,renderer.layer(&device,&queue,&grey,1280,720,[0.,0.,1280.,720.])).unwrap();
  assert!(bg.ready_frame(2));
  bg.update(&renderer,&device,&queue,2);
  assert_eq!(bg.layers(2).len(),1);
 });
}
