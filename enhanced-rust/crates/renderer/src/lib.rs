//! Portable texture compositor. Shader sources are offline Slang output.
use wgpu::util::DeviceExt;
pub mod vector;
pub mod font;
pub mod control_pack;
pub mod background;
#[repr(C)]
#[derive(Clone,Copy,bytemuck::Pod,bytemuck::Zeroable)]
pub struct Vertex{pub pos:[f32;2],pub uv:[f32;2],pub opacity:[f32;2]}
pub struct Layer {pub binding:wgpu::BindGroup,pub vertices:wgpu::Buffer,pub texture:wgpu::Texture,pub count:u32}
pub struct Renderer {pipeline:wgpu::RenderPipeline,layout:wgpu::BindGroupLayout,sampler:wgpu::Sampler}
impl Renderer{
    pub fn new(device:&wgpu::Device,format:wgpu::TextureFormat,vertex:&str,fragment:&str,vs_entry:&str,fs_entry:&str)->Self{
        let vs=device.create_shader_module(wgpu::ShaderModuleDescriptor{label:Some("vertex"),source:wgpu::ShaderSource::Wgsl(vertex.into())});
        let fs=device.create_shader_module(wgpu::ShaderModuleDescriptor{label:Some("fragment"),source:wgpu::ShaderSource::Wgsl(fragment.into())});
        let layout=device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{label:None,entries:&[
            wgpu::BindGroupLayoutEntry{binding:0,visibility:wgpu::ShaderStages::FRAGMENT,ty:wgpu::BindingType::Texture{sample_type:wgpu::TextureSampleType::Float{filterable:true},view_dimension:wgpu::TextureViewDimension::D2,multisampled:false},count:None},
            wgpu::BindGroupLayoutEntry{binding:1,visibility:wgpu::ShaderStages::FRAGMENT,ty:wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),count:None},
            wgpu::BindGroupLayoutEntry{binding:2,visibility:wgpu::ShaderStages::FRAGMENT,ty:wgpu::BindingType::Texture{sample_type:wgpu::TextureSampleType::Float{filterable:true},view_dimension:wgpu::TextureViewDimension::D2,multisampled:false},count:None}
        ]});
        let pl=device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{label:None,bind_group_layouts:&[&layout],push_constant_ranges:&[]});
        let pipeline=device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
            label:Some("presskit compositor"),layout:Some(&pl),
            vertex:wgpu::VertexState{module:&vs,entry_point:Some(vs_entry),compilation_options:Default::default(),buffers:&[wgpu::VertexBufferLayout{array_stride:24,step_mode:wgpu::VertexStepMode::Vertex,attributes:&wgpu::vertex_attr_array![0=>Float32x2,2=>Float32x2,1=>Float32x2]}]},
            fragment:Some(wgpu::FragmentState{module:&fs,entry_point:Some(fs_entry),compilation_options:Default::default(),targets:&[Some(wgpu::ColorTargetState{format,blend:Some(wgpu::BlendState::ALPHA_BLENDING),write_mask:wgpu::ColorWrites::ALL})]}),
            primitive:Default::default(),depth_stencil:None,multisample:Default::default(),multiview:None,cache:None});
        let sampler=device.create_sampler(&wgpu::SamplerDescriptor{mag_filter:wgpu::FilterMode::Linear,min_filter:wgpu::FilterMode::Linear,..Default::default()});
        Self{pipeline,layout,sampler}
    }
    pub fn layer(&self,device:&wgpu::Device,queue:&wgpu::Queue,rgba:&[u8],width:u32,height:u32,rect:[f32;4])->Layer{
        let vertices=self.vertices(device,rect);
        let size=wgpu::Extent3d{width,height,depth_or_array_layers:1};
        let texture=self.texture(device,size);
        queue.write_texture(wgpu::TexelCopyTextureInfo{texture:&texture,mip_level:0,origin:wgpu::Origin3d::ZERO,aspect:wgpu::TextureAspect::All},rgba,wgpu::TexelCopyBufferLayout{offset:0,bytes_per_row:Some(width*4),rows_per_image:Some(height)},size);
        self.bind(device,texture,vertices)
    }
    fn vertices(&self,device:&wgpu::Device,rect:[f32;4])->wgpu::Buffer{
        let verts=Self::quad(rect,[1.,-1.]);
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor{label:None,contents:bytemuck::cast_slice(&verts),usage:wgpu::BufferUsages::VERTEX|wgpu::BufferUsages::COPY_DST})
    }
    fn quad(rect:[f32;4],opacity:[f32;2])->[Vertex;6]{
        let [x,y,w,h]=rect;
        let p=|x:f32,y:f32|[x/640.-1.,1.-y/360.];
        let a=Vertex{pos:p(x,y),uv:[0.,0.],opacity};let b=Vertex{pos:p(x+w,y),uv:[1.,0.],opacity};
        let c=Vertex{pos:p(x,y+h),uv:[0.,1.],opacity};let d=Vertex{pos:p(x+w,y+h),uv:[1.,1.],opacity};
        [a,c,b,b,c,d]
    }
    fn texture(&self,device:&wgpu::Device,size:wgpu::Extent3d)->wgpu::Texture{
        device.create_texture(&wgpu::TextureDescriptor{label:None,size,mip_level_count:1,sample_count:1,dimension:wgpu::TextureDimension::D2,format:wgpu::TextureFormat::Rgba8UnormSrgb,usage:wgpu::TextureUsages::TEXTURE_BINDING|wgpu::TextureUsages::COPY_DST|wgpu::TextureUsages::RENDER_ATTACHMENT,view_formats:&[]})
    }
    fn bind(&self,device:&wgpu::Device,texture:wgpu::Texture,vertices:wgpu::Buffer)->Layer{
        let view=texture.create_view(&Default::default());
        let binding=device.create_bind_group(&wgpu::BindGroupDescriptor{label:None,layout:&self.layout,entries:&[
            wgpu::BindGroupEntry{binding:0,resource:wgpu::BindingResource::TextureView(&view)},
            wgpu::BindGroupEntry{binding:1,resource:wgpu::BindingResource::Sampler(&self.sampler)},
            wgpu::BindGroupEntry{binding:2,resource:wgpu::BindingResource::TextureView(&view)}
        ]});Layer{binding,vertices,texture,count:6}
    }
    pub fn polygon(&self,device:&wgpu::Device,queue:&wgpu::Queue,rgba:[u8;4],points:&[[f32;2]])->Option<Layer>{
        let triangles=vector::triangulate(points)?;
        self.mesh(device,queue,rgba,&triangles)
    }
    pub fn mesh(&self,device:&wgpu::Device,queue:&wgpu::Queue,rgba:[u8;4],triangles:&[[f32;2]])->Option<Layer>{
        if triangles.is_empty()||triangles.len()%3!=0{return None;}
        let verts:Vec<_>=triangles.iter().map(|p|Vertex{pos:[p[0]/640.-1.,1.-p[1]/360.],uv:[0.5,0.5],opacity:[1.,-1.]}).collect();
        let mut layer=self.layer(device,queue,&rgba,1,1,[0.,0.,1.,1.]);
        layer.vertices=device.create_buffer_init(&wgpu::util::BufferInitDescriptor{label:Some("vector mesh"),contents:bytemuck::cast_slice(&verts),usage:wgpu::BufferUsages::VERTEX});layer.count=verts.len() as u32;Some(layer)
    }
    pub fn background_pair(&self,device:&wgpu::Device,first:&Layer,second:&Layer)->Layer{
        let a=first.texture.create_view(&Default::default());let b=second.texture.create_view(&Default::default());
        let binding=device.create_bind_group(&wgpu::BindGroupDescriptor{label:Some("original background pair"),layout:&self.layout,entries:&[
            wgpu::BindGroupEntry{binding:0,resource:wgpu::BindingResource::TextureView(&a)},wgpu::BindGroupEntry{binding:1,resource:wgpu::BindingResource::Sampler(&self.sampler)},wgpu::BindGroupEntry{binding:2,resource:wgpu::BindingResource::TextureView(&b)}]});
        Layer{binding,vertices:self.vertices(device,[0.,0.,1280.,720.]),texture:first.texture.clone(),count:6}
    }
    pub fn set_background_opacity(&self,queue:&wgpu::Queue,layer:&Layer,alpha:[f32;2]){queue.write_buffer(&layer.vertices,0,bytemuck::cast_slice(&Self::quad([0.,0.,1280.,720.],alpha)));}
    pub fn set_opacity(&self,queue:&wgpu::Queue,layer:&Layer,rect:[f32;4],opacity:f32){queue.write_buffer(&layer.vertices,0,bytemuck::cast_slice(&Self::quad(rect,[opacity,-1.])));}
    pub fn reposition(&self,device:&wgpu::Device,layer:&mut Layer,rect:[f32;4]){layer.vertices=self.vertices(device,rect);}
    #[cfg(target_arch="wasm32")]
    pub fn external_layer(&self,device:&wgpu::Device,queue:&wgpu::Queue,source:wgpu::ExternalImageSource,rect:[f32;4])->Layer{
        let size=wgpu::Extent3d{width:source.width(),height:source.height(),depth_or_array_layers:1};
        let texture=self.texture(device,size);
        let layer=self.bind(device,texture,self.vertices(device,rect));
        Self::copy_external(queue,&layer,source);layer
    }
    #[cfg(target_arch="wasm32")]
    pub fn copy_external(queue:&wgpu::Queue,layer:&Layer,source:wgpu::ExternalImageSource){
        let size=wgpu::Extent3d{width:source.width(),height:source.height(),depth_or_array_layers:1};
        queue.copy_external_image_to_texture(&wgpu::CopyExternalImageSourceInfo{source,origin:wgpu::Origin2d::ZERO,flip_y:false},wgpu::CopyExternalImageDestInfo{texture:&layer.texture,mip_level:0,origin:wgpu::Origin3d::ZERO,aspect:wgpu::TextureAspect::All,color_space:wgpu::PredefinedColorSpace::Srgb,premultiplied_alpha:false},size);
    }
    pub fn render(&self,encoder:&mut wgpu::CommandEncoder,target:&wgpu::TextureView,layers:&[&Layer],viewport:[f32;4]){
        let mut pass=encoder.begin_render_pass(&wgpu::RenderPassDescriptor{label:None,color_attachments:&[Some(wgpu::RenderPassColorAttachment{view:target,resolve_target:None,ops:wgpu::Operations{load:wgpu::LoadOp::Clear(wgpu::Color::BLACK),store:wgpu::StoreOp::Store}})],depth_stencil_attachment:None,timestamp_writes:None,occlusion_query_set:None});
        let [x,y,w,h]=viewport;pass.set_viewport(x,y,w,h,0.,1.);pass.set_pipeline(&self.pipeline);
        for layer in layers{pass.set_bind_group(0,&layer.binding,&[]);pass.set_vertex_buffer(0,layer.vertices.slice(..));pass.draw(0..layer.count,0..1);}
    }
}
