//! The centralized management of video sub-system.

use std::sync::{Arc, RwLock};
use std::time::Duration;

use application::window::Window;
use math;
use utils::object_pool;

use super::assets::prelude::*;
use super::batch::DrawCall;
use super::errors::*;

use super::backends::frame::*;
use super::backends::gl::visitor::GLVisitor;
use super::backends::Visitor;

/// The information of video module during last frame.
#[derive(Debug, Copy, Clone, Default)]
pub struct VideoFrameInfo {
    pub duration: Duration,
    pub drawcall: u32,
    pub triangles: u32,
    pub alive_surfaces: u32,
    pub alive_shaders: u32,
    pub alive_meshes: u32,
    pub alive_textures: u32,
}

/// The centralized management of video sub-system.
pub struct VideoSystem {
    visitor: Box<Visitor>,
    frames: Arc<DoubleFrame>,
    shared: Arc<VideoSystemShared>,
    last_dimensions: math::Vector2<u32>,
}

impl VideoSystem {
    /// Create a new `VideoSystem` with one `Window` context.
    pub fn new(window: &Window) -> ::errors::Result<Self> {
        let frames = Arc::new(DoubleFrame::with_capacity(64 * 1024));
        let shared = VideoSystemShared::new(frames.clone());
        let visitor = unsafe { Box::new(GLVisitor::glutin(window)?) };

        Ok(VideoSystem {
            last_dimensions: window.dimensions(),
            visitor: visitor,

            frames: frames,
            shared: Arc::new(shared),
        })
    }

    /// Returns the multi-thread friendly parts of `VideoSystem`.
    pub fn shared(&self) -> Arc<VideoSystemShared> {
        self.shared.clone()
    }

    /// Swap internal commands frame.
    #[inline]
    pub fn swap_frames(&self) {
        self.frames.swap_frames();
    }

    /// Advance to next frame.
    ///
    /// Notes that this method MUST be called at main thread, and will NOT return
    /// until all commands is finished by GPU.
    pub fn advance(&mut self, window: &Window) -> ::errors::Result<VideoFrameInfo> {
        use std::time;

        let ts = time::Instant::now();
        let dimensions = window.dimensions();

        // Resize the window, which would recreate the underlying framebuffer.
        if dimensions != self.last_dimensions {
            self.last_dimensions = dimensions;
            window.resize(dimensions);
        }

        let (dc, tris) = self.frames
            .back()
            .dispatch(self.visitor.as_mut(), dimensions)?;
        let mut info = VideoFrameInfo::default();

        {
            let s = &self.shared;
            info.alive_surfaces = s.surfaces.write().unwrap().len() as u32;
            info.alive_shaders = s.shaders.write().unwrap().len() as u32;
            info.alive_meshes = s.meshes.write().unwrap().len() as u32;
            info.alive_textures = s.textures.write().unwrap().len() as u32;
            info.drawcall = dc;
            info.triangles = tris;
        }

        info.duration = time::Instant::now() - ts;
        Ok(info)
    }
}

enum AsyncState<T> {
    Ok(T),
    NotReady,
}

/// The multi-thread friendly parts of `VideoSystem`.
pub struct VideoSystemShared {
    pub(crate) frames: Arc<DoubleFrame>,

    textures: RwLock<object_pool::ObjectPool<AsyncState<()>>>,
    surfaces: RwLock<object_pool::ObjectPool<SurfaceParams>>,
    shaders: RwLock<object_pool::ObjectPool<ShaderParams>>,
    render_textures: RwLock<object_pool::ObjectPool<RenderTextureParams>>,
    meshes: RwLock<object_pool::ObjectPool<AsyncState<MeshParams>>>,
}

impl VideoSystemShared {
    /// Create a new `VideoSystem` with one `Window` context.
    fn new(frames: Arc<DoubleFrame>) -> Self {
        VideoSystemShared {
            frames: frames,

            surfaces: RwLock::new(object_pool::ObjectPool::new()),
            shaders: RwLock::new(object_pool::ObjectPool::new()),
            meshes: RwLock::new(object_pool::ObjectPool::new()),
            textures: RwLock::new(object_pool::ObjectPool::new()),
            render_textures: RwLock::new(object_pool::ObjectPool::new()),
        }
    }

    /// Draws ur mesh.
    ///
    /// Notes that you should use [Batch](crate::video::batch::Batch) if possible.
    #[inline]
    pub fn draw(&self, handle: SurfaceHandle, dc: DrawCall) {
        let mut frame = self.frames.front();
        let len = dc.uniforms_len;
        let ptr = frame.bufs.extend_from_slice(&dc.uniforms[0..len]);
        let cmd = Command::Draw(dc.shader, dc.mesh, dc.mesh_index, ptr);

        frame.cmds.push(Command::Bind(handle));
        frame.cmds.push(cmd);
    }

    /// Updates the scissor test of surface.
    ///
    /// The test is initially disabled. While the test is enabled, only pixels that lie within
    /// the scissor box can be modified by drawing commands.
    ///
    /// Notes that you should use [Batch](crate::video::batch::Batch) if possible.
    #[inline]
    pub fn update_scissor(&self, handle: SurfaceHandle, scissor: SurfaceScissor) {
        let mut frame = self.frames.front();
        frame.cmds.push(Command::Bind(handle));
        frame.cmds.push(Command::UpdateScissor(scissor));
    }

    /// Updates the scissor test of surface.
    ///
    /// The test is initially disabled. While the test is enabled, only pixels that lie within
    /// the scissor box can be modified by drawing commands.
    ///
    /// Notes that you should use [Batch](crate::video::batch::Batch) if possible.
    #[inline]
    pub fn update_viewport(&self, handle: SurfaceHandle, viewport: SurfaceViewport) {
        let mut frame = self.frames.front();
        frame.cmds.push(Command::Bind(handle));
        frame.cmds.push(Command::UpdateViewport(viewport));
    }
}

impl VideoSystemShared {
    /// Creates an view with `SurfaceParams`.
    pub fn create_surface(&self, params: SurfaceParams) -> Result<SurfaceHandle> {
        let handle = self.surfaces.write().unwrap().create(params).into();

        {
            let cmd = Command::CreateSurface(handle, params);
            self.frames.front().cmds.push(cmd);
        }

        Ok(handle)
    }

    /// Gets the `SurfaceParams` if available.
    pub fn surface(&self, handle: SurfaceHandle) -> Option<SurfaceParams> {
        self.surfaces.read().unwrap().get(handle).cloned()
    }

    /// Deletes surface object.
    pub fn delete_surface(&self, handle: SurfaceHandle) {
        if self.surfaces.write().unwrap().free(handle).is_some() {
            let cmd = Command::DeleteSurface(handle);
            self.frames.front().cmds.push(cmd);
        }
    }
}

impl VideoSystemShared {
    /// Create a shader with initial shaders and render state. It encapusulates all the
    /// informations we need to configurate graphics pipeline before real drawing.
    pub fn create_shader(
        &self,
        params: ShaderParams,
        vs: String,
        fs: String,
    ) -> Result<ShaderHandle> {
        params.validate(&vs, &fs)?;

        let handle = self.shaders.write().unwrap().create(params.clone()).into();

        {
            let cmd = Command::CreateShader(handle, params, vs, fs);
            self.frames.front().cmds.push(cmd);
        }

        Ok(handle)
    }

    /// Gets the `ShaderParams` if available.
    pub fn shader(&self, handle: MeshHandle) -> Option<ShaderParams> {
        self.shaders.read().unwrap().get(handle).cloned()
    }

    /// Delete shader state object.
    pub fn delete_shader(&self, handle: ShaderHandle) {
        if self.shaders.write().unwrap().free(handle).is_some() {
            let cmd = Command::DeleteShader(handle);
            self.frames.front().cmds.push(cmd);
        }
    }
}

impl VideoSystemShared {
    /// Create a new mesh object.
    pub fn create_mesh<T>(&self, params: MeshParams, data: T) -> Result<MeshHandle>
    where
        T: Into<Option<MeshData>>,
    {
        let data = data.into();
        params.validate(data.as_ref())?;

        let handle = self.meshes
            .write()
            .unwrap()
            .create(AsyncState::Ok(params.clone()))
            .into();

        {
            let mut frame = self.frames.front();
            let cmd = Command::CreateMesh(handle, params, data);
            frame.cmds.push(cmd);
        }

        Ok(handle)
    }

    /// Gets the `MeshParams` if available.
    pub fn mesh_aabb(&self, handle: MeshHandle) -> Option<math::Aabb3<f32>> {
        self.meshes.read().unwrap().get(handle).and_then(|v| {
            if let AsyncState::Ok(v) = v {
                Some(v.aabb)
            } else {
                None
            }
        })
    }

    /// Update a subset of dynamic vertex buffer. Use `offset` specifies the offset
    /// into the buffer object's data store where data replacement will begin, measured
    /// in bytes.
    pub fn update_vertex_buffer(
        &self,
        handle: MeshHandle,
        offset: usize,
        data: &[u8],
    ) -> Result<()> {
        if let Some(_) = self.meshes.read().unwrap().get(handle) {
            let mut frame = self.frames.front();
            let ptr = frame.bufs.extend_from_slice(data);
            let cmd = Command::UpdateVertexBuffer(handle, offset, ptr);
            frame.cmds.push(cmd);
            Ok(())
        } else {
            Err(Error::HandleInvalid(format!("{:?}", handle)))
        }
    }

    /// Update a subset of dynamic index buffer. Use `offset` specifies the offset
    /// into the buffer object's data store where data replacement will begin, measured
    /// in bytes.
    pub fn update_index_buffer(
        &self,
        handle: MeshHandle,
        offset: usize,
        data: &[u8],
    ) -> Result<()> {
        if let Some(_) = self.meshes.read().unwrap().get(handle) {
            let mut frame = self.frames.front();
            let ptr = frame.bufs.extend_from_slice(data);
            let cmd = Command::UpdateIndexBuffer(handle, offset, ptr);
            frame.cmds.push(cmd);

            Ok(())
        } else {
            Err(Error::HandleInvalid(format!("{:?}", handle)))
        }
    }

    /// Delete mesh object.
    pub fn delete_mesh(&self, handle: MeshHandle) {
        if self.meshes.write().unwrap().free(handle).is_some() {
            let cmd = Command::DeleteMesh(handle);
            self.frames.front().cmds.push(cmd);
        }
    }

    pub(crate) fn loader_create_mesh(&self) -> Result<MeshHandle> {
        let handle = self.meshes
            .write()
            .unwrap()
            .create(AsyncState::NotReady)
            .into();

        Ok(handle)
    }

    pub(crate) fn loader_update_mesh(
        &self,
        handle: MeshHandle,
        params: MeshParams,
        data: MeshData,
    ) -> Result<()> {
        params.validate(Some(&data))?;

        if let Some(v) = self.meshes.write().unwrap().get_mut(handle) {
            let mut frame = self.frames.front();
            let task = Command::CreateMesh(handle, params.clone(), Some(data));
            frame.cmds.push(task);
            *v = AsyncState::Ok(params);

            Ok(())
        } else {
            Err(Error::HandleInvalid(format!("{:?}", handle)))
        }
    }
}

impl VideoSystemShared {
    /// Create texture object. A texture is an image loaded in video memory,
    /// which can be sampled in shaders.
    pub fn create_texture<'a, T>(&self, params: TextureParams, data: T) -> Result<TextureHandle>
    where
        T: Into<Option<TextureData>>,
    {
        let data = data.into();
        params.validate(data.as_ref())?;

        let handle = self.textures
            .write()
            .unwrap()
            .create(AsyncState::Ok(()))
            .into();

        {
            let mut frame = self.frames.front();
            let task = Command::CreateTexture(handle, params, data);
            frame.cmds.push(task);
        }

        Ok(handle)
    }

    /// Update a contiguous subregion of an existing two-dimensional texture object.
    pub fn update_texture(
        &self,
        handle: TextureHandle,
        area: math::Aabb2<u32>,
        data: &[u8],
    ) -> Result<()> {
        if let Some(AsyncState::Ok(_)) = self.textures.read().unwrap().get(handle) {
            let mut frame = self.frames.front();
            let ptr = frame.bufs.extend_from_slice(data);
            let cmd = Command::UpdateTexture(handle, area, ptr);
            frame.cmds.push(cmd);

            Ok(())
        } else {
            Err(Error::HandleInvalid(format!("{:?}", handle)))
        }
    }

    /// Delete the texture object.
    pub fn delete_texture(&self, handle: TextureHandle) {
        if self.textures.write().unwrap().free(handle).is_some() {
            let cmd = Command::DeleteTexture(handle);
            self.frames.front().cmds.push(cmd);
        }
    }

    pub(crate) fn loader_create_texture(&self) -> Result<TextureHandle> {
        let handle = self.textures
            .write()
            .unwrap()
            .create(AsyncState::NotReady)
            .into();

        Ok(handle)
    }

    pub(crate) fn loader_update_texture(
        &self,
        handle: TextureHandle,
        params: TextureParams,
        data: TextureData,
    ) -> Result<()> {
        params.validate(Some(&data))?;

        if let Some(v) = self.textures.write().unwrap().get_mut(handle) {
            let mut frame = self.frames.front();
            let task = Command::CreateTexture(handle, params, Some(data));
            frame.cmds.push(task);
            *v = AsyncState::Ok(());

            Ok(())
        } else {
            Err(Error::HandleInvalid(format!("{:?}", handle)))
        }
    }
}

impl VideoSystemShared {
    /// Create render texture object, which could be attached with a framebuffer.
    pub fn create_render_texture(
        &self,
        params: RenderTextureParams,
    ) -> Result<RenderTextureHandle> {
        let handle = self.render_textures.write().unwrap().create(params).into();

        {
            let cmd = Command::CreateRenderTexture(handle, params);
            self.frames.front().cmds.push(cmd);
        }

        Ok(handle)
    }

    /// Gets the `RenderTextureParams` if available.
    pub fn render_texture(&self, handle: RenderTextureHandle) -> Option<RenderTextureParams> {
        self.render_textures.read().unwrap().get(handle).cloned()
    }

    /// Delete the render texture object.
    pub fn delete_render_texture(&self, handle: RenderTextureHandle) {
        if self.render_textures.write().unwrap().free(handle).is_some() {
            let cmd = Command::DeleteRenderTexture(handle);
            self.frames.front().cmds.push(cmd);
        }
    }
}
