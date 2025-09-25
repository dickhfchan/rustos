#![allow(dead_code)]

use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use crate::memory;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsError {
    InvalidFramebuffer,
    UnsupportedFormat,
    OutOfMemory,
    InvalidDimensions,
}

pub type GraphicsResult<T> = Result<T, GraphicsError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    RGB888,
    RGBA8888,
    BGR888,
    BGRA8888,
    RGB565,
}

#[derive(Debug)]
pub struct Framebuffer {
    width: u32,
    height: u32,
    format: PixelFormat,
    stride: u32,
    buffer: usize,
    size: usize,
}

#[derive(Debug)]
pub struct Surface {
    id: u32,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    format: PixelFormat,
    buffer: Option<usize>,
    visible: bool,
    z_order: i32,
}

#[derive(Debug)]
pub struct Window {
    id: u32,
    title: &'static str,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    surface: Option<u32>,
    focused: bool,
    minimized: bool,
    maximized: bool,
}

#[derive(Debug)]
pub struct Compositor {
    framebuffer: Option<Framebuffer>,
    surfaces: BTreeMap<u32, Surface>,
    windows: BTreeMap<u32, Window>,
    next_surface_id: u32,
    next_window_id: u32,
    focused_window: Option<u32>,
}

static mut COMPOSITOR: Option<Compositor> = None;

impl PixelFormat {
    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            PixelFormat::RGB888 => 3,
            PixelFormat::RGBA8888 => 4,
            PixelFormat::BGR888 => 3,
            PixelFormat::BGRA8888 => 4,
            PixelFormat::RGB565 => 2,
        }
    }
}

impl Framebuffer {
    pub fn new(width: u32, height: u32, format: PixelFormat) -> GraphicsResult<Self> {
        if width == 0 || height == 0 {
            return Err(GraphicsError::InvalidDimensions);
        }

        let bytes_per_pixel = format.bytes_per_pixel();
        let stride = width * bytes_per_pixel;
        let size = (stride * height) as usize;
        
        let buffer = memory::allocate_pages(size)
            .map_err(|_| GraphicsError::OutOfMemory)?;

        Ok(Framebuffer {
            width,
            height,
            format,
            stride,
            buffer: buffer as usize,
            size,
        })
    }

    pub fn get_buffer_ptr(&self) -> *mut u8 {
        self.buffer as *mut u8
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    pub fn get_stride(&self) -> u32 {
        self.stride
    }

    pub fn get_format(&self) -> PixelFormat {
        self.format
    }

    pub fn clear(&mut self, color: u32) -> GraphicsResult<()> {
        let ptr = self.get_buffer_ptr();
        let bytes_per_pixel = self.format.bytes_per_pixel();
        
        unsafe {
            for y in 0..self.height {
                for x in 0..self.width {
                    let offset = ((y * self.stride) + (x * bytes_per_pixel)) as isize;
                    let pixel_ptr = ptr.offset(offset) as *mut u32;
                    *pixel_ptr = color;
                }
            }
        }
        Ok(())
    }

    pub fn draw_pixel(&mut self, x: u32, y: u32, color: u32) -> GraphicsResult<()> {
        if x >= self.width || y >= self.height {
            return Err(GraphicsError::InvalidDimensions);
        }

        let ptr = self.get_buffer_ptr();
        let bytes_per_pixel = self.format.bytes_per_pixel();
        let offset = ((y * self.stride) + (x * bytes_per_pixel)) as isize;
        
        unsafe {
            let pixel_ptr = ptr.offset(offset) as *mut u32;
            *pixel_ptr = color;
        }
        
        Ok(())
    }

    pub fn draw_rectangle(&mut self, x: u32, y: u32, width: u32, height: u32, color: u32) -> GraphicsResult<()> {
        for dy in 0..height {
            for dx in 0..width {
                if x + dx < self.width && y + dy < self.height {
                    self.draw_pixel(x + dx, y + dy, color)?;
                }
            }
        }
        Ok(())
    }
}

impl Surface {
    pub fn new(id: u32, width: u32, height: u32, format: PixelFormat) -> GraphicsResult<Self> {
        Ok(Surface {
            id,
            x: 0,
            y: 0,
            width,
            height,
            format,
            buffer: None,
            visible: true,
            z_order: 0,
        })
    }

    pub fn set_position(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub fn set_size(&mut self, width: u32, height: u32) -> GraphicsResult<()> {
        if width == 0 || height == 0 {
            return Err(GraphicsError::InvalidDimensions);
        }
        self.width = width;
        self.height = height;
        Ok(())
    }

    pub fn attach_buffer(&mut self, buffer: usize) {
        self.buffer = Some(buffer);
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn set_z_order(&mut self, z_order: i32) {
        self.z_order = z_order;
    }

    pub fn get_bounds(&self) -> (i32, i32, u32, u32) {
        (self.x, self.y, self.width, self.height)
    }
}

impl Window {
    pub fn new(id: u32, title: &'static str, x: i32, y: i32, width: u32, height: u32) -> Self {
        Window {
            id,
            title,
            x,
            y,
            width,
            height,
            surface: None,
            focused: false,
            minimized: false,
            maximized: false,
        }
    }

    pub fn attach_surface(&mut self, surface_id: u32) {
        self.surface = Some(surface_id);
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn set_minimized(&mut self, minimized: bool) {
        self.minimized = minimized;
    }

    pub fn set_maximized(&mut self, maximized: bool) {
        self.maximized = maximized;
    }

    pub fn move_window(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub fn resize_window(&mut self, width: u32, height: u32) -> GraphicsResult<()> {
        if width == 0 || height == 0 {
            return Err(GraphicsError::InvalidDimensions);
        }
        self.width = width;
        self.height = height;
        Ok(())
    }

    pub fn get_bounds(&self) -> (i32, i32, u32, u32) {
        (self.x, self.y, self.width, self.height)
    }

    pub fn is_visible(&self) -> bool {
        !self.minimized
    }
}

impl Compositor {
    pub fn new() -> Self {
        Compositor {
            framebuffer: None,
            surfaces: BTreeMap::new(),
            windows: BTreeMap::new(),
            next_surface_id: 1,
            next_window_id: 1,
            focused_window: None,
        }
    }

    pub fn initialize_framebuffer(&mut self, width: u32, height: u32, format: PixelFormat) -> GraphicsResult<()> {
        let fb = Framebuffer::new(width, height, format)?;
        self.framebuffer = Some(fb);
        Ok(())
    }

    pub fn create_surface(&mut self, width: u32, height: u32, format: PixelFormat) -> GraphicsResult<u32> {
        let surface_id = self.next_surface_id;
        self.next_surface_id += 1;

        let surface = Surface::new(surface_id, width, height, format)?;
        self.surfaces.insert(surface_id, surface);
        Ok(surface_id)
    }

    pub fn create_window(&mut self, title: &'static str, x: i32, y: i32, width: u32, height: u32) -> GraphicsResult<u32> {
        let window_id = self.next_window_id;
        self.next_window_id += 1;

        let window = Window::new(window_id, title, x, y, width, height);
        self.windows.insert(window_id, window);
        Ok(window_id)
    }

    pub fn get_surface_mut(&mut self, surface_id: u32) -> Option<&mut Surface> {
        self.surfaces.get_mut(&surface_id)
    }

    pub fn get_window_mut(&mut self, window_id: u32) -> Option<&mut Window> {
        self.windows.get_mut(&window_id)
    }

    pub fn attach_surface_to_window(&mut self, window_id: u32, surface_id: u32) -> GraphicsResult<()> {
        if !self.surfaces.contains_key(&surface_id) {
            return Err(GraphicsError::InvalidFramebuffer);
        }

        if let Some(window) = self.windows.get_mut(&window_id) {
            window.attach_surface(surface_id);
            Ok(())
        } else {
            Err(GraphicsError::InvalidFramebuffer)
        }
    }

    pub fn set_window_focus(&mut self, window_id: Option<u32>) -> GraphicsResult<()> {
        // Unfocus current window
        if let Some(current_focus) = self.focused_window {
            if let Some(window) = self.windows.get_mut(&current_focus) {
                window.set_focused(false);
            }
        }

        // Focus new window
        if let Some(window_id) = window_id {
            if let Some(window) = self.windows.get_mut(&window_id) {
                window.set_focused(true);
                self.focused_window = Some(window_id);
            } else {
                return Err(GraphicsError::InvalidFramebuffer);
            }
        } else {
            self.focused_window = None;
        }

        Ok(())
    }

    pub fn composite(&mut self) -> GraphicsResult<()> {
        // Clear the framebuffer
        if let Some(ref mut fb) = self.framebuffer {
            fb.clear(0x000000)?; // Black background
        } else {
            return Err(GraphicsError::InvalidFramebuffer);
        }

        // Collect window data to avoid borrow checker issues
        let mut window_data: Vec<_> = self.windows.iter()
            .map(|(_, w)| (w.id, w.x, w.y, w.width, w.height, w.focused, w.minimized, w.surface))
            .collect();
        window_data.sort_by_key(|(id, _, _, _, _, _, _, _)| *id); // Simple ordering for now

        for (_, x, y, width, height, focused, minimized, surface_id) in window_data {
            if minimized {
                continue;
            }

            if let Some(surface_id) = surface_id {
                if let Some(surface) = self.surfaces.get(&surface_id) {
                    if surface.visible {
                        // Draw the window directly here to avoid borrowing issues
                        self.draw_window_direct(x, y, width, height, focused)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn draw_window_direct(&mut self, x: i32, y: i32, width: u32, height: u32, focused: bool) -> GraphicsResult<()> {
        let fb = self.framebuffer.as_mut().ok_or(GraphicsError::InvalidFramebuffer)?;
        
        // Draw window frame
        let frame_color = if focused { 0x4A90E2 } else { 0x7F7F7F };
        let title_bar_height = 30;
        
        // Title bar
        fb.draw_rectangle(
            x as u32,
            y as u32,
            width,
            title_bar_height,
            frame_color
        )?;

        // Window border
        let border_width = 2;
        fb.draw_rectangle(
            x as u32,
            (y + title_bar_height as i32) as u32,
            width,
            border_width,
            frame_color
        )?;

        // Content area (simplified - just fill with white)
        fb.draw_rectangle(
            x as u32,
            (y + title_bar_height as i32 + border_width as i32) as u32,
            width,
            height - title_bar_height - border_width,
            0xFFFFFF
        )?;

        Ok(())
    }

    fn draw_window(&self, fb: &mut Framebuffer, window: &Window, surface: &Surface) -> GraphicsResult<()> {
        // Draw window frame
        let frame_color = if window.focused { 0x4A90E2 } else { 0x7F7F7F };
        let title_bar_height = 30;
        
        // Title bar
        fb.draw_rectangle(
            window.x as u32,
            window.y as u32,
            window.width,
            title_bar_height,
            frame_color
        )?;

        // Window border
        let border_width = 2;
        fb.draw_rectangle(
            window.x as u32,
            (window.y + title_bar_height as i32) as u32,
            window.width,
            border_width,
            frame_color
        )?;

        // Content area (simplified - just fill with white)
        fb.draw_rectangle(
            window.x as u32,
            (window.y + title_bar_height as i32 + border_width as i32) as u32,
            window.width,
            window.height - title_bar_height - border_width,
            0xFFFFFF
        )?;

        Ok(())
    }

    pub fn get_framebuffer(&self) -> Option<&Framebuffer> {
        self.framebuffer.as_ref()
    }

    pub fn get_window_count(&self) -> usize {
        self.windows.len()
    }

    pub fn get_surface_count(&self) -> usize {
        self.surfaces.len()
    }
}

// Public API functions
pub fn graphics_init() -> GraphicsResult<()> {
    unsafe {
        if COMPOSITOR.is_some() {
            return Err(GraphicsError::InvalidFramebuffer);
        }
        
        let compositor = Compositor::new();
        COMPOSITOR = Some(compositor);
    }
    Ok(())
}

pub fn graphics_get_compositor() -> Option<&'static mut Compositor> {
    unsafe { COMPOSITOR.as_mut() }
}

pub fn graphics_init_framebuffer(width: u32, height: u32, format: PixelFormat) -> GraphicsResult<()> {
    let compositor = graphics_get_compositor().ok_or(GraphicsError::InvalidFramebuffer)?;
    compositor.initialize_framebuffer(width, height, format)
}

pub fn graphics_create_window(title: &'static str, x: i32, y: i32, width: u32, height: u32) -> GraphicsResult<u32> {
    let compositor = graphics_get_compositor().ok_or(GraphicsError::InvalidFramebuffer)?;
    compositor.create_window(title, x, y, width, height)
}

pub fn graphics_create_surface(width: u32, height: u32, format: PixelFormat) -> GraphicsResult<u32> {
    let compositor = graphics_get_compositor().ok_or(GraphicsError::InvalidFramebuffer)?;
    compositor.create_surface(width, height, format)
}

pub fn graphics_attach_surface_to_window(window_id: u32, surface_id: u32) -> GraphicsResult<()> {
    let compositor = graphics_get_compositor().ok_or(GraphicsError::InvalidFramebuffer)?;
    compositor.attach_surface_to_window(window_id, surface_id)
}

pub fn graphics_set_window_focus(window_id: Option<u32>) -> GraphicsResult<()> {
    let compositor = graphics_get_compositor().ok_or(GraphicsError::InvalidFramebuffer)?;
    compositor.set_window_focus(window_id)
}

pub fn graphics_composite() -> GraphicsResult<()> {
    let compositor = graphics_get_compositor().ok_or(GraphicsError::InvalidFramebuffer)?;
    compositor.composite()
}

pub fn graphics_get_stats() -> (usize, usize) {
    if let Some(compositor) = graphics_get_compositor() {
        (compositor.get_window_count(), compositor.get_surface_count())
    } else {
        (0, 0)
    }
}