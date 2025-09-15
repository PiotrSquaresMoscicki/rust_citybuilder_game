pub mod rendering_device;
pub mod rendering_manager;
pub mod web_client_rendering_device;
pub mod web_service_manager;
// pub mod rendering2d_system;

pub use rendering_device::{RenderingDevice, RenderCommand, RenderResult};
pub use rendering_manager::{initialize_global_rendering_manager, get_global_rendering_manager, render_global_grid};
pub use web_client_rendering_device::WebClientRenderingDevice;
pub use web_service_manager::WebServiceManager;
// pub use rendering2d_system::{Rendering2dSystem, rendering2d_system, VisibleSprite, VisibleShape, RenderableEntity};