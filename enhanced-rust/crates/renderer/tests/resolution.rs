use presskit_renderer::Renderer;
#[test]
fn native_subpixel_geometry_is_rasterized_at_1440p_and_4k(){
 pollster::block_on(async{
  let instance=wgpu::Instance::default();
  let adapter=instance.request_adapter(&Default::default()).await.expect("GPU adapter required");
  let mut desc=wgpu::DeviceDescriptor::default();
  desc.required_limits.max_texture_dimension_2d=adapter.limits().max_texture_dimension_2d;
  let(d,q)=adapter.request_device(&desc,None).await.unwrap();
  let r=Renderer::new(&d,&q,wgpu::TextureFormat::Rgba8UnormSrgb,include_str!("../../../shaders/generated/vertex.wgsl"),include_str!("../../../shaders/generated/fragment.wgsl"),"vertexMain","fragmentMain");
  // These two stripes are each narrower than a single logical stage pixel.
  // A 720p raster upscale cannot reproduce the alternating dark/light samples.
  let mut triangles=vec![];
  for (x0,x1) in [(100.0,100.4),(101.0,101.4)]{
   triangles.extend([[x0,100.],[x0,120.],[x1,100.],[x1,100.],[x0,120.],[x1,120.]]);
  }
  let layer=r.mesh(&d,&q,[255;4],&triangles).unwrap();
  for scale in [2u32,3,2]{
   let(w,h)=(1280*scale,720*scale);
   assert!(w<=d.limits().max_texture_dimension_2d,"adapter cannot support the requested test resolution");
   let size=wgpu::Extent3d{width:w,height:h,depth_or_array_layers:1};
   let target=d.create_texture(&wgpu::TextureDescriptor{label:Some("physical resolution test"),size,mip_level_count:1,sample_count:1,dimension:wgpu::TextureDimension::D2,format:wgpu::TextureFormat::Rgba8UnormSrgb,usage:wgpu::TextureUsages::RENDER_ATTACHMENT|wgpu::TextureUsages::COPY_SRC,view_formats:&[]});
   // Copy just one physical row from the rendered target to avoid full 4K readback.
   let buf=d.create_buffer(&wgpu::BufferDescriptor{label:Some("physical pixel row"),size:u64::from(w)*4,usage:wgpu::BufferUsages::COPY_DST|wgpu::BufferUsages::MAP_READ,mapped_at_creation:false});
   let mut enc=d.create_command_encoder(&Default::default());
   r.render(&mut enc,&target.create_view(&Default::default()),&[&layer],[0.,0.,w as f32,h as f32]);
   enc.copy_texture_to_buffer(wgpu::TexelCopyTextureInfo{texture:&target,mip_level:0,origin:wgpu::Origin3d{x:0,y:110*scale,z:0},aspect:wgpu::TextureAspect::All},wgpu::TexelCopyBufferInfo{buffer:&buf,layout:wgpu::TexelCopyBufferLayout{offset:0,bytes_per_row:Some(w*4),rows_per_image:Some(1)}},wgpu::Extent3d{width:w,height:1,depth_or_array_layers:1});
   q.submit([enc.finish()]);let slice=buf.slice(..);let(tx,rx)=std::sync::mpsc::channel();slice.map_async(wgpu::MapMode::Read,move|v|tx.send(v).unwrap());d.poll(wgpu::Maintain::Wait);rx.recv().unwrap().unwrap();let pixels=slice.get_mapped_range();
   let red=|x:u32|pixels[x as usize*4];
   for start in [100*scale,101*scale]{
    assert!(red(start)>250,"{w}x{h}: native subpixel stripe disappeared");
    for gap in 1..scale{assert!(red(start+gap)<5,"{w}x{h}: stripe blurred into adjacent pixels");}
   }
   assert!(red(100*scale-1)<5);eprintln!("{w}x{h}: native subpixel geometry and resize readback passed");
   drop(pixels);buf.unmap();
  }
 });
}
