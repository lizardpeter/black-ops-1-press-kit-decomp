//! Portable texture compositor. Initial development shader is explicit fallback;
//! compile Slang with tools/compile_shaders.py before using the production path.
use wgpu::util::DeviceExt;
#[repr(C)]
#[derive(Clone,Copy,bytemuck::Pod,bytemuck::Zeroable)]
pub struct Vertex{pub pos:[f32;2],pub uv:[f32;2]}
pub struct Layer {pub binding:wgpu::BindGroup,pub vertices:wgpu::Buffer}
pub struct Renderer {pipeline:wgpu::RenderPipeline,layout:wgpu::BindGroupLayout,sampler:wgpu::Sampler}
impl Renderer{
    pub fn new(device:&wgpu::Device,format:wgpu::TextureFormat,vertex:&str,fragment:&str,vs_entry:&str,fs_entry:&str)->Self{
        let vs=device.create_shader_module(wgpu::ShaderModuleDescriptor{label:Some("vertex"),source:wgpu::ShaderSource::Wgsl(vertex.into())});
        let fs=device.create_shader_module(wgpu::ShaderModuleDescriptor{label:Some("fragment"),source:wgpu::ShaderSource::Wgsl(fragment.into())});
        let layout=device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{label:None,entries:&[
            wgpu::BindGroupLayoutEntry{binding:0,visibility:wgpu::ShaderStages::FRAGMENT,ty:wgpu::BindingType::Texture{sample_type:wgpu::TextureSampleType::Float{filterable:true},view_dimension:wgpu::TextureViewDimension::D2,multisampled:false},count:None},
            wgpu::BindGroupLayoutEntry{binding:1,visibility:wgpu::ShaderStages::FRAGMENT,ty:wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),count:None}
        ]});
        let pl=device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{label:None,bind_group_layouts:&[&layout],push_constant_ranges:&[]});
        let pipeline=device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
            label:Some("presskit compositor"),layout:Some(&pl),
            vertex:wgpu::VertexState{module:&vs,entry_point:Some(vs_entry),compilation_options:Default::default(),buffers:&[wgpu::VertexBufferLayout{array_stride:16,step_mode:wgpu::VertexStepMode::Vertex,attributes:&wgpu::vertex_attr_array![0=>Float32x2,1=>Float32x2]}]},
            fragment:Some(wgpu::FragmentState{module:&fs,entry_point:Some(fs_entry),compilation_options:Default::default(),targets:&[Some(wgpu::ColorTargetState{format,blend:Some(wgpu::BlendState::ALPHA_BLENDING),write_mask:wgpu::ColorWrites::ALL})]}),
            primitive:Default::default(),depth_stencil:None,multisample:Default::default(),multiview:None,cache:None});
        let sampler=device.create_sampler(&wgpu::SamplerDescriptor{mag_filter:wgpu::FilterMode::Linear,min_filter:wgpu::FilterMode::Linear,..Default::default()});
        Self{pipeline,layout,sampler}
    }
    pub fn layer(&self,device:&wgpu::Device,queue:&wgpu::Queue,rgba:&[u8],width:u32,height:u32,rect:[f32;4])->Layer{
        let [x,y,w,h]=rect;
        let p=|x:f32,y:f32|[x/640.-1.,1.-y/360.];
        let a=Vertex{pos:p(x,y),uv:[0.,0.]};let b=Vertex{pos:p(x+w,y),uv:[1.,0.]};
        let c=Vertex{pos:p(x,y+h),uv:[0.,1.]};let d=Vertex{pos:p(x+w,y+h),uv:[1.,1.]};
        let vertices=device.create_buffer_init(&wgpu::util::BufferInitDescriptor{label:None,contents:bytemuck::cast_slice(&[a,c,b,b,c,d]),usage:wgpu::BufferUsages::VERTEX});
        let size=wgpu::Extent3d{width,height,depth_or_array_layers:1};
        let texture=device.create_texture(&wgpu::TextureDescriptor{label:None,size,mip_level_count:1,sample_count:1,dimension:wgpu::TextureDimension::D2,format:wgpu::TextureFormat::Rgba8UnormSrgb,usage:wgpu::TextureUsages::TEXTURE_BINDING|wgpu::TextureUsages::COPY_DST,view_formats:&[]});
        queue.write_texture(wgpu::TexelCopyTextureInfo{texture:&texture,mip_level:0,origin:wgpu::Origin3d::ZERO,aspect:wgpu::TextureAspect::All},rgba,wgpu::TexelCopyBufferLayout{offset:0,bytes_per_row:Some(width*4),rows_per_image:Some(height)},size);
        let view=texture.create_view(&Default::default());
        let binding=device.create_bind_group(&wgpu::BindGroupDescriptor{label:None,layout:&self.layout,entries:&[
            wgpu::BindGroupEntry{binding:0,resource:wgpu::BindingResource::TextureView(&view)},
            wgpu::BindGroupEntry{binding:1,resource:wgpu::BindingResource::Sampler(&self.sampler)}
        ]});Layer{binding,vertices}
    }
    pub fn render(&self,encoder:&mut wgpu::CommandEncoder,target:&wgpu::TextureView,layers:&[&Layer],viewport:[f32;4]){
        let mut pass=encoder.begin_render_pass(&wgpu::RenderPassDescriptor{label:None,color_attachments:&[Some(wgpu::RenderPassColorAttachment{view:target,resolve_target:None,ops:wgpu::Operations{load:wgpu::LoadOp::Clear(wgpu::Color::BLACK),store:wgpu::StoreOp::Store}})],depth_stencil_attachment:None,timestamp_writes:None,occlusion_query_set:None});
        let [x,y,w,h]=viewport;pass.set_viewport(x,y,w,h,0.,1.);pass.set_pipeline(&self.pipeline);
        for layer in layers{pass.set_bind_group(0,&layer.binding,&[]);pass.set_vertex_buffer(0,layer.vertices.slice(..));pass.draw(0..6,0..1);}
    }
}
