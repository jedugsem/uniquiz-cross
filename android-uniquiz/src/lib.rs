mod clipboard;
//mod controls;
mod java;
mod scene;
use crate::winit::event::ElementState;
use crate::winit::event_loop::ActiveEventLoop;
use crate::winit::event_loop::EventLoopProxy;
use crate::winit::keyboard::KeyCode;
use crate::winit::keyboard::NamedKey;
use crate::winit::keyboard::PhysicalKey;
use crate::winit::platform::android::activity::AndroidApp;
use clipboard::Clipboard;
use iced_core::clipboard::Kind;
use iced_core::Clipboard as ClipInterface;
use iced_wgpu::graphics::Viewport;
use iced_wgpu::{wgpu, Engine, Renderer};
use iced_winit::conversion;
use iced_winit::core::mouse;
use iced_winit::core::renderer;
use iced_winit::core::time::Instant;
use iced_winit::core::window;
use iced_winit::core::{Event, Font, Pixels, Size, Theme};
use iced_winit::futures;
use iced_winit::runtime::user_interface::{self, UserInterface};
use iced_winit::winit;
use iced_winit::winit::platform::android::EventLoopBuilderExtAndroid;
use log::LevelFilter;
use scene::Scene;
use std::sync::Arc;
use uniquiz::{Controls, Message, UserEvent};
use winit::{
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    keyboard::ModifiersState,
};
// winit ime support
// https://github.com/rust-windowing/winit/pull/2993

// issue with android-activity crate default_motion_filter function
// https://github.com/rust-mobile/android-activity/issues/79

#[no_mangle]
fn android_main(android_app: AndroidApp) {
    let logger_config = android_logger::Config::default().with_max_level(LevelFilter::Info);
    android_logger::init_once(logger_config);

    log::info!("android_main started");

    let event_loop = EventLoop::with_user_event()
        .with_android_app(android_app)
        .build()
        .expect("Should build event loop");

    let proxy = event_loop.create_proxy();

    let mut runner = Runner::Loading(proxy.clone());
    event_loop
        .run_app(&mut runner)
        .expect("Should run event loop");
    android_intent::with_current_env(|env| {
        android_intent::Intent::new(env, "Settings.ACTION_MANAGE_APP_ALL_FILE_ACCES_PERMISSION")
            .with_type("android.intent.category.DEFAULT")
            .with_extra(android_intent::Extra::Text, "package:com.jedugsem.uniquiz")
            .start_activity()
            .unwrap()
    });
}

#[allow(clippy::large_enum_variant)]
enum Runner {
    Loading(EventLoopProxy<UserEvent>),
    Ready(Ready),
}
struct Ready {
    proxy: EventLoopProxy<UserEvent>,
    window: Arc<winit::window::Window>,
    queue: wgpu::Queue,
    device: wgpu::Device,
    surface: wgpu::Surface<'static>,
    format: wgpu::TextureFormat,
    renderer: Renderer,
    scene: Scene,
    controls: Controls,
    events: Vec<Event>,
    cursor: mouse::Cursor,
    cache: user_interface::Cache,
    clipboard: Clipboard,
    viewport: Viewport,
    modifiers: ModifiersState,
    resized: bool,
}

impl winit::application::ApplicationHandler<UserEvent> for Runner {
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::ShowKeyboard => {
                java::call_instance_method("showKeyboard");
            }
            UserEvent::HideKeyboard => {
                java::call_instance_method("hideKeyboard");
            }
            UserEvent::Back => {
                if let Runner::Ready(Ready {
                    controls, window, ..
                }) = self
                {
                    _ = controls.update(Message::Back);
                    window.request_redraw();
                }
            }
            UserEvent::Task(message) => {
                if let Runner::Ready(Ready {
                    controls, window, ..
                }) = self
                {
                    _ = controls.update(message);
                    window.request_redraw();
                }
            }
            UserEvent::ClipboardWrite(m) => {
                if let Runner::Ready(Ready {
                    clipboard, window, ..
                }) = self
                {
                    clipboard.write(Kind::Standard, m);
                    window.request_redraw();
                }
            }
            UserEvent::ClipboardRead(num) => {
                if let Runner::Ready(Ready {
                    controls,
                    window,
                    clipboard,
                    ..
                }) = self
                {
                    if let Some(m) = clipboard.read(Kind::Standard) {
                        _ = controls.update(Message::Clipboard(m, num));
                        window.request_redraw();
                    }
                }
            }
        }
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        // let (proxy, controls) = match self {
        //     Self::Loading(proxy) => (proxy.clone(), Controls::new(proxy.clone())),
        //     Self::Ready {
        //         proxy, controls, ..
        //     } => (proxy.clone(), controls.clone()),
        // };
        let window = Arc::new(
            event_loop
                .create_window(winit::window::WindowAttributes::default())
                .expect("Create window"),
        );

        let physical_size = window.inner_size();
        let viewport = Viewport::with_physical_size(
            Size::new(physical_size.width, physical_size.height),
            window.scale_factor() * 1.4,
        );
        // TODO
        let clipboard = Clipboard {};

        let backend = wgpu::Backends::from_env().unwrap_or_default();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: backend,
            ..Default::default()
        });
        let surface = instance
            .create_surface(window.clone())
            .expect("Create window surface");

        let (format, adapter, device, queue) = futures::futures::executor::block_on(async {
            let adapter =
                wgpu::util::initialize_adapter_from_env_or_default(&instance, Some(&surface))
                    .await
                    .expect("Create adapter");

            let adapter_features = adapter.features();

            let capabilities = surface.get_capabilities(&adapter);

            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        required_features: adapter_features & wgpu::Features::default(),
                        required_limits: wgpu::Limits::default(),
                        memory_hints: wgpu::MemoryHints::MemoryUsage,
                    },
                    None,
                )
                .await
                .expect("Request device");

            (
                capabilities
                    .formats
                    .iter()
                    .copied()
                    .find(wgpu::TextureFormat::is_srgb)
                    .or_else(|| capabilities.formats.first().copied())
                    .expect("Get preferred format"),
                adapter,
                device,
                queue,
            )
        });

        surface.configure(
            &device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format,
                width: physical_size.width,
                height: physical_size.height,
                present_mode: wgpu::PresentMode::AutoVsync,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            },
        );

        // Initialize scene and GUI controls
        let scene = Scene::new(&device, format);

        // Initialize iced

        let renderer = {
            let engine = Engine::new(&adapter, device.clone(), queue.clone(), format, None);

            Renderer::new(engine, Font::default(), Pixels::from(16))
        };

        // You should change this if you want to render continuously
        event_loop.set_control_flow(ControlFlow::Wait);
        match self {
            Self::Ready(ready) => {
                ready.window = window;
                ready.device = device;
                ready.queue = queue;
                ready.renderer = renderer;
                ready.surface = surface;
                ready.format = format;
                ready.scene = scene;
                ready.events = Vec::new();
                ready.cursor = mouse::Cursor::Unavailable;
                ready.modifiers = ModifiersState::default();
                ready.cache = user_interface::Cache::new();
                ready.clipboard = clipboard;
                ready.viewport = viewport;
                ready.resized = false;
            }
            Self::Loading(proxy) => {
                *self = Self::Ready(Ready {
                    proxy: proxy.clone(),
                    window,
                    device,
                    queue,
                    renderer,
                    surface,
                    format,
                    scene,
                    controls: Controls::new(proxy.clone()),
                    events: Vec::new(),
                    cursor: mouse::Cursor::Unavailable,
                    modifiers: ModifiersState::default(),
                    cache: user_interface::Cache::new(),
                    clipboard,
                    viewport,
                    resized: false,
                });
            }
        }
        if let Runner::Ready(Ready {
            controls, window, ..
        }) = self
        {
            _ = controls.update(Message::Boot);
            window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Self::Ready(Ready {
            proxy,
            window,
            device,
            queue,
            surface,
            format,
            renderer,
            scene,
            controls,
            events,
            viewport,
            cursor,
            modifiers,
            clipboard,
            cache,
            resized,
        }) = self
        else {
            return;
        };

        match event {
            WindowEvent::RedrawRequested => {
                if *resized {
                    let size = window.inner_size();

                    *viewport = Viewport::with_physical_size(
                        Size::new(size.width, size.height),
                        window.scale_factor() * 1.4,
                    );

                    surface.configure(
                        device,
                        &wgpu::SurfaceConfiguration {
                            format: *format,
                            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                            width: size.width,
                            height: size.height,
                            present_mode: wgpu::PresentMode::AutoVsync,
                            alpha_mode: wgpu::CompositeAlphaMode::Auto,
                            view_formats: vec![],
                            desired_maximum_frame_latency: 2,
                        },
                    );

                    *resized = false;
                }

                match surface.get_current_texture() {
                    Ok(frame) => {
                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        let mut encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: None,
                            });

                        {
                            // Clear the frame
                            let mut render_pass =
                                Scene::clear(&view, &mut encoder, controls.background_color());

                            // Draw the scene
                            scene.draw(&mut render_pass);
                        }

                        // Submit the scene
                        queue.submit([encoder.finish()]);

                        // Draw iced on top
                        let mut interface = UserInterface::build(
                            controls.view(),
                            viewport.logical_size(),
                            std::mem::take(cache),
                            renderer,
                        );

                        let _ = interface.update(
                            &[Event::Window(
                                window::Event::RedrawRequested(Instant::now()),
                            )],
                            *cursor,
                            renderer,
                            clipboard,
                            &mut Vec::new(),
                        );

                        let mouse_interaction = interface.draw(
                            renderer,
                            &Theme::Dark,
                            &renderer::Style::default(),
                            *cursor,
                        );
                        *cache = interface.into_cache();

                        renderer.present(None, frame.texture.format(), &view, viewport);

                        // Present the frame
                        frame.present();

                        // Update the mouse cursor
                        window.set_cursor(conversion::mouse_interaction(mouse_interaction));
                    }
                    Err(error) => match error {
                        wgpu::SurfaceError::OutOfMemory => {
                            panic!(
                                "Swapchain error: {error}. \
                                        Rendering cannot continue."
                            )
                        }
                        _ => {
                            // Try rendering again next frame.
                            window.request_redraw();
                        }
                    },
                }
            }
            WindowEvent::Touch(touch) => {
                *cursor = mouse::Cursor::Available(conversion::cursor_position(
                    touch.location,
                    viewport.scale_factor(),
                ));
            }
            WindowEvent::CursorMoved { position, .. } => {
                *cursor = mouse::Cursor::Available(conversion::cursor_position(
                    position,
                    viewport.scale_factor(),
                ));
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                ref event,
                is_synthetic: _,
            } => {
                match event.logical_key {
                    winit::keyboard::Key::Named(NamedKey::GoBack) => {
                        _ = proxy.send_event(UserEvent::Back);
                    }
                    _ => {}
                }
                if let PhysicalKey::Code(code) = event.physical_key {
                    match code {
                        KeyCode::BrowserBack => {
                            _ = proxy.send_event(UserEvent::Back);
                        }
                        KeyCode::ShiftLeft | KeyCode::ShiftRight => match event.state {
                            ElementState::Pressed => *modifiers |= ModifiersState::SHIFT,
                            ElementState::Released => *modifiers &= !ModifiersState::SHIFT,
                        },
                        KeyCode::ControlLeft | KeyCode::ControlRight => match event.state {
                            ElementState::Pressed => *modifiers |= ModifiersState::CONTROL,
                            ElementState::Released => *modifiers &= !ModifiersState::CONTROL,
                        },
                        _ => (),
                    }
                }
            }

            WindowEvent::ModifiersChanged(new_modifiers) => {
                *modifiers = new_modifiers.state();
            }
            WindowEvent::Resized(_) => {
                *resized = true;
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }

        // Map window event to iced event
        if let Some(event) = conversion::window_event(event, window.scale_factor(), *modifiers) {
            events.push(event);
        }

        // If there are events pending
        if !events.is_empty() {
            // We process them
            let mut interface = UserInterface::build(
                controls.view(),
                viewport.logical_size(),
                std::mem::take(cache),
                renderer,
            );

            let mut messages = Vec::new();

            let _ = interface.update(events, *cursor, renderer, clipboard, &mut messages);

            events.clear();
            *cache = interface.into_cache();

            // update our UI with any messages
            for message in messages {
                _ = controls.update(message);
            }

            // and request a redraw
            window.request_redraw();
        }
    }
}
