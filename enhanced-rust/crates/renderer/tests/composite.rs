use presskit_renderer::Renderer;
#[test]
fn generated_slang_composites_pixels() {
    pollster::block_on(async {
        let instance=wgpu::Instance::default();
        let adapter=instance.request_adapter(&wgpu::RequestAdapterOptions::default()).await.expect("headless GPU adapter");
        eprintln!("adapter: {:?}",adapter.get_info());
        let (device,queue)=adapter.request_device(&wgpu::DeviceDescriptor::default(),None).await.unwrap();
        let renderer=Renderer::new(&device,wgpu::TextureFormat::Rgba8UnormSrgb,
            include_str!("../../../shaders/generated/vertex.wgsl"),include_str!("../../../shaders/generated/fragment.wgsl"),"vertexMain","fragmentMain");
        let bg=renderer.layer(&device,&queue,&[255,0,0,255],1,1,[0.,0.,1280.,720.]);
        let fg=renderer.layer(&device,&queue,&[0,255,0,255],1,1,[640.,0.,640.,720.]);
        let size=wgpu::Extent3d{width:64,height:4,depth_or_array_layers:1};
        let target=device.create_texture(&wgpu::TextureDescriptor{label:None,size,mip_level_count:1,sample_count:1,dimension:wgpu::TextureDimension::D2,format:wgpu::TextureFormat::Rgba8UnormSrgb,usage:wgpu::TextureUsages::RENDER_ATTACHMENT|wgpu::TextureUsages::COPY_SRC,view_formats:&[]});
        let buffer=device.create_buffer(&wgpu::BufferDescriptor{label:None,size:1024,usage:wgpu::BufferUsages::COPY_DST|wgpu::BufferUsages::MAP_READ,mapped_at_creation:false});
        let mut encoder=device.create_command_encoder(&Default::default());
        renderer.render(&mut encoder,&target.create_view(&Default::default()),&[&bg,&fg],[0.,0.,64.,4.]);
        encoder.copy_texture_to_buffer(wgpu::TexelCopyTextureInfo{texture:&target,mip_level:0,origin:wgpu::Origin3d::ZERO,aspect:wgpu::TextureAspect::All},wgpu::TexelCopyBufferInfo{buffer:&buffer,layout:wgpu::TexelCopyBufferLayout{offset:0,bytes_per_row:Some(256),rows_per_image:Some(4)}},size);
        queue.submit([encoder.finish()]);
        let slice=buffer.slice(..);let (tx,rx)=std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read,move|r|tx.send(r).unwrap());
        device.poll(wgpu::Maintain::Wait);rx.recv().unwrap().unwrap();
        let data=slice.get_mapped_range();
        assert_eq!(&data[0..4],&[255,0,0,255]);
        assert_eq!(&data[128..132],&[0,255,0,255]);
    });
}

#[test]
fn original_control_pack_gpu() {
    let Ok(path)=std::env::var("PRESSKIT_CONTROL_PACK") else {eprintln!("set PRESSKIT_CONTROL_PACK to test source-derived asset");return;};
    let bytes=std::fs::read(path).unwrap();let pack=presskit_renderer::control_pack::ControlPack::read(&bytes).unwrap();
    pollster::block_on(async{
        let instance=wgpu::Instance::default();let adapter=instance.request_adapter(&wgpu::RequestAdapterOptions::default()).await.unwrap();let (device,queue)=adapter.request_device(&wgpu::DeviceDescriptor::default(),None).await.unwrap();
        let renderer=Renderer::new(&device,wgpu::TextureFormat::Rgba8UnormSrgb,include_str!("../../../shaders/generated/vertex.wgsl"),include_str!("../../../shaders/generated/fragment.wgsl"),"vertexMain","fragmentMain");
        let size=wgpu::Extent3d{width:1280,height:720,depth_or_array_layers:1};
        let target=device.create_texture(&wgpu::TextureDescriptor{label:None,size,mip_level_count:1,sample_count:1,dimension:wgpu::TextureDimension::D2,format:wgpu::TextureFormat::Rgba8UnormSrgb,usage:wgpu::TextureUsages::RENDER_ATTACHMENT|wgpu::TextureUsages::COPY_SRC,view_formats:&[]});
        let buffer=device.create_buffer(&wgpu::BufferDescriptor{label:None,size:1280*720*4,usage:wgpu::BufferUsages::COPY_DST|wgpu::BufferUsages::MAP_READ,mapped_at_creation:false});
        for gallery in [false,true]{for state in 0..3{
            let layers=pack.layers(&renderer,&device,&queue,&presskit_renderer::control_pack::ControlVisual{y:620.,gallery,paused:false,progress:0.4,text:if gallery{"20 / 20"}else{"00:04/01:42"}.into(),states:[state;4]});assert!(layers.len()>6);
            let mut encoder=device.create_command_encoder(&Default::default());renderer.render(&mut encoder,&target.create_view(&Default::default()),&layers.iter().collect::<Vec<_>>(),[0.,0.,1280.,720.]);
            encoder.copy_texture_to_buffer(wgpu::TexelCopyTextureInfo{texture:&target,mip_level:0,origin:wgpu::Origin3d::ZERO,aspect:wgpu::TextureAspect::All},wgpu::TexelCopyBufferInfo{buffer:&buffer,layout:wgpu::TexelCopyBufferLayout{offset:0,bytes_per_row:Some(5120),rows_per_image:Some(720)}},size);queue.submit([encoder.finish()]);
            let slice=buffer.slice(..);let (tx,rx)=std::sync::mpsc::channel();slice.map_async(wgpu::MapMode::Read,move|r|tx.send(r).unwrap());device.poll(wgpu::Maintain::Wait);rx.recv().unwrap().unwrap();let data=slice.get_mapped_range();
            let x=if gallery{300}else{225};let off=(625*1280+x)*4;
            if !gallery{assert_eq!(&data[off..off+4],&[[242,180,7,255],[198,139,6,255],[107,75,3,255]][state as usize]);}else{assert!(data[off]>80);}
            drop(data);buffer.unmap();
        }}
    });
}
