use phys_engine::{engine::{Instance, Vertex}, SIDE_LENGTH};
use winit::{dpi::PhysicalSize, event::{self, ElementState, Event, KeyEvent, WindowEvent}, event_loop::EventLoop, keyboard::{KeyCode, PhysicalKey}, window::WindowBuilder};

fn main() {
    pollster::block_on(run())
}

async fn run()
{
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().with_inner_size(PhysicalSize::new(200, 200)).build(&event_loop).unwrap();
    let mut instance = Instance::new(&window).await;

    let _ = event_loop.run(move |event, control_flow| {
        match event
        {
            Event::WindowEvent { window_id, event } if !instance.input(&event) => match event
            {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            ..
                        },
                    ..
                } => control_flow.exit(),
                WindowEvent::Resized(new_size) =>
                {
                    instance.resize(new_size);                   
                },
                WindowEvent::RedrawRequested =>
                {
                    instance.window().set_title(&format!("Rendering {} particles at {} fps", SIDE_LENGTH * SIDE_LENGTH, instance.estimate_fps()));
                    instance.window().request_redraw();
                    instance.update();
                    
                    match instance.render()
                    {
                        Ok(_) => {
                        },
                        Err(
                            wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated
                        ) => instance.reconfig(),
                        Err(
                            wgpu::SurfaceError::OutOfMemory
                        ) => {
                            log::error!("Out of memory");
                            control_flow.exit()
                        },
                        Err(wgpu::SurfaceError::Timeout) =>
                        {
                            log::warn!("Timeout")
                        }
                    }
                }
                _ => {}
            },
            _ => {},
        }
    });
}