use std::{cell::RefCell, collections::{HashMap, VecDeque}, rc::Rc};

use femtovg::{Canvas, renderer::OpenGl};
use glutin::{ContextBuilder, event::{ElementState, VirtualKeyCode}, event_loop::{ControlFlow, EventLoop, EventLoopProxy}, window::{WindowBuilder, WindowId}};
use morphorm::{Units, PositionType};

use crate::{AppData, BoundingBox, CachedData, Color, Context, Data, Display, Entity, Enviroment, Event, EventManager, FontOrId, IdManager, ModelData, Modifiers, MouseButton, MouseButtonState, MouseState, Propagation, ResourceManager, Style, Tree, TreeExt, Visibility, Window, WindowDescription, WindowEvent, apply_hover, apply_styles, geometry_changed, scan_to_code, style::apply_transform, style_system::{apply_clipping, apply_visibility, apply_z_ordering}, vcode_to_code, vk_to_key, RawWindow};

static DEFAULT_THEME: &str = include_str!("default_theme.css");

#[derive(Debug, Clone, Copy)]
pub enum AppEvent {
    CreateWindow,
}

pub struct Application {
    context: Context,
    event_loop: EventLoop<Event>,
    builder: Option<Box<dyn Fn(&mut Context)>>,
    on_idle: Option<Box<dyn Fn(&mut Context)>>,
    window_description: WindowDescription,
    should_poll: bool,
    primary_window: Entity,
}

impl Application {
    pub fn new<F>(window_description: WindowDescription, builder: F) -> Self
    where F: 'static + Fn(&mut Context)
    {

        let mut entity_manager = IdManager::new();
        // Create the root
        entity_manager.create();
        
        let mut tree = Tree::new();

        let primary_window = entity_manager.create();
        tree.add(primary_window, Entity::root());

        let mut cache = CachedData::default();
        cache.add(Entity::root()).expect("Failed to add entity to cache");
        cache.add(primary_window).expect("Failed to add entity to cache");

        let mut context = Context {
            entity_manager,
            tree,
            current: primary_window,
            count: 0,
            views: HashMap::new(),
            lenses: HashMap::new(),
            //state: HashMap::new(),  
            data: AppData::new(),
            style: Rc::new(RefCell::new(Style::default())),
            cache,
            enviroment: Enviroment::new(),
            event_queue: VecDeque::new(),
            mouse: MouseState::default(),
            modifiers: Modifiers::empty(),
            captured: Entity::null(),
            hovered: primary_window,
            focused: primary_window,
            //state_count: 0,
            resource_manager: ResourceManager::new(),
            fonts: Vec::new(),
        };

        

        context.style.borrow_mut().add(primary_window);



        context.add_theme(DEFAULT_THEME);

        Self {
            context,
            event_loop: EventLoop::with_user_event(),
            builder: Some(Box::new(builder)),
            on_idle: None,
            window_description,
            should_poll: false,
            primary_window,
        }
    }


    pub fn should_poll(mut self) -> Self {
        self.should_poll = true;

        self
    }


    /// Takes a closure which will be called at the end of every loop of the application.
    /// 
    /// The callback provides a place to run 'idle' processing and happens at the end of each loop but before drawing.
    /// If the callback pushes events into the queue in state then the event loop will re-run. Care must be taken not to
    /// push events into the queue every time the callback runs unless this is intended.
    /// 
    /// # Example
    /// ```
    /// Application::new(WindowDescription::new(), |state, window|{
    ///     // Build application here
    /// })
    /// .on_idle(|state|{
    ///     // Code here runs at the end of every event loop after OS and tuix events have been handled 
    /// })
    /// .run();
    /// ```
    pub fn on_idle<F: 'static + Fn(&mut Context)>(mut self, callback: F) -> Self {
        self.on_idle = Some(Box::new(callback));

        self
    } 

    // TODO - Rename this
    pub fn get_proxy(&self) -> EventLoopProxy<Event> {
        self.event_loop.create_proxy()
    }

    pub fn background_color(self, color: Color) -> Self {
        self.context.style.borrow_mut().background_color.insert(self.primary_window, color);

        self
    }

    pub fn locale(mut self, id: &str) -> Self {
        self.context.enviroment.set_locale(id);


        self
    }

    pub fn run(mut self) {

        let mut context = self.context;

        let primary_window = self.primary_window;

        println!("Primary Window: {}", primary_window);
        
        let event_loop = self.event_loop;
        
        // let handle = ContextBuilder::new()
        //     .with_vsync(true)
        //     .build_windowed(WindowBuilder::new(), &event_loop)
        //     .expect("Failed to build windowed context");

        // let handle = unsafe { handle.make_current().unwrap() };

        // let renderer = OpenGl::new(|s| handle.context().get_proc_address(s) as *const _)
        //     .expect("Cannot create renderer");
        // let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");


        let mut window = Window::new(&event_loop, &self.window_description);

        // let font = canvas.add_font_mem(FONT).expect("Failed to load font");

        // context.fonts = vec![font];

        let regular_font = include_bytes!("../fonts/Roboto-Regular.ttf");
        let bold_font = include_bytes!("../fonts/Roboto-Bold.ttf");
        let icon_font = include_bytes!("../fonts/entypo.ttf");
        let emoji_font = include_bytes!("../fonts/OpenSansEmoji.ttf");
        let arabic_font = include_bytes!("../fonts/amiri-regular.ttf");

        context.add_font_mem("roboto", regular_font);
        context.add_font_mem("roboto-bold", bold_font);
        context.add_font_mem("icon", icon_font);
        context.add_font_mem("emoji", emoji_font);
        context.add_font_mem("arabic", arabic_font);

        context.style.borrow_mut().default_font = "roboto".to_string();

        let dpi_factor = window.handle.window().scale_factor();
        let size = window.handle.window().inner_size();

        let clear_color = context.style.borrow_mut().background_color.get(primary_window).cloned().unwrap_or_default();

        window.canvas.set_size(size.width as u32, size.height as u32, dpi_factor as f32);
        window.canvas.clear_rect(
            0,
            0,
            size.width as u32,
            size.height as u32,
            clear_color.into(),
        );

        let mut windows: HashMap<WindowId, Entity> = HashMap::new();

        windows.insert(window.id, primary_window);

        context.views.insert(primary_window, Box::new(window));

        context
            .cache
            .set_width(primary_window, self.window_description.inner_size.width as f32);
        context
            .cache
            .set_height(primary_window, self.window_description.inner_size.height as f32);

        context.style.borrow_mut().width.insert(primary_window, Units::Pixels(self.window_description.inner_size.width as f32));
        context.style.borrow_mut().height.insert(primary_window, Units::Pixels(self.window_description.inner_size.height as f32));

        context.style.borrow_mut().position_type.insert(primary_window, PositionType::SelfDirected);

        let mut bounding_box = BoundingBox::default();
        bounding_box.w = size.width as f32;
        bounding_box.h = size.height as f32;

        context.cache.set_clip_region(primary_window, bounding_box);

        let mut event_manager = EventManager::new();

        // if let Some(builder) = self.builder.take() {
        //     (builder)(&mut context);

        //     self.builder = Some(builder);
        // }

        let builder = self.builder.take();

        let on_idle = self.on_idle.take();

        let event_loop_proxy = event_loop.create_proxy();

        let should_poll = self.should_poll;

        event_loop.run(move |event, event_loop_window_target, control_flow|{

            if should_poll {
                *control_flow = ControlFlow::Poll;
            } else {
                *control_flow = ControlFlow::Wait;
            }

            match event {

                glutin::event::Event::UserEvent(event) => {
                    context.event_queue.push_back(event);
                }

                glutin::event::Event::MainEventsCleared => {

                    if context.enviroment.needs_rebuild {
                        context.current = primary_window;
                        context.count = 0;
                        if let Some(builder) = &builder {
                            (builder)(&mut context);
                        }
                        context.enviroment.needs_rebuild = false;
                    }

                    if let Some(mut window_view) = context.views.remove(&primary_window) {
                        if let Some(window) = window_view.downcast_mut::<Window>() {


                            // Load resources
                            for (name, font) in context.resource_manager.fonts.iter_mut() {
                    
                                match font {
                                    FontOrId::Font(data) => {
                                        let id1 = window.canvas.add_font_mem(&data.clone()).expect(&format!("Failed to load font file for: {}", name));
                                        //let id2 = context.text_context.add_font_mem(&data.clone()).expect("failed");
                                        // if id1 != id2 {
                                        //     panic!("Fonts in canvas must have the same id as fonts in the text context");
                                        // }
                                        *font = FontOrId::Id(id1);
                                    }
                    
                                    _=> {}
                                }
                            }

                        }

                        context.views.insert(primary_window, window_view);
                    }

                    // Events
                    while !context.event_queue.is_empty() {
                        let mut app_events = event_manager.flush_events(&mut context);
                        for event in app_events.iter_mut() {
                            if let Some(app_event) = event.message.downcast() {
                                match app_event {
                                    AppEvent::CreateWindow => {
                                        println!("Create another window: {}", event.origin);
                                        if let Some(mut view) = context.views.remove(&event.origin) {
                                            if let Some(raw_window) = view.downcast_mut::<RawWindow>() {
                                                println!("Right here");
                                                let window_id = raw_window.create(&mut context, event_loop_window_target);
                                                windows.insert(window_id, event.origin);
                                            }

                                            context.views.insert(event.origin, view);
                                        }
                                    }
                                }
                            }
                        }
                        
                    }

                    // Data Updates
                    let mut observers: Vec<Entity> = Vec::new();
                    for model_list in context.data.model_data.dense.iter().map(|entry| &entry.value){
                        for (_, model) in model_list.iter() {
                            //println!("Lenses: {:?}", context.lenses.len());
                            for (_, lens) in context.lenses.iter_mut() {
                                if lens.update(model) {
                                    observers.extend(lens.observers().iter());
                                }
                            }
                        }
                    }

                    for observer in observers.iter() {
                        if let Some(mut view) = context.views.remove(observer) {
                            let prev = context.current;
                            context.current = *observer;
                            let prev_count = context.count;
                            context.count = 0;
                            view.body(&mut context);
                            context.current = prev;
                            context.count = prev_count;
                
            
                            context.views.insert(*observer, view);
                        }
                    }

                    context.current = primary_window;

                    // Not ideal
                    let tree = context.tree.clone();

                    // Styling
                    apply_styles(&mut context, &tree, primary_window);


                    apply_z_ordering(&mut context, &tree, primary_window);

                    apply_visibility(&mut context, &tree, primary_window);

                    // Layout
                    if context.style.borrow().needs_relayout {
                        morphorm::layout(&mut context.cache, &context.tree, &context.style.borrow());
                        context.style.borrow_mut().needs_relayout = false;
                    }

                    // Emit any geometry changed events
                    geometry_changed(&mut context, &tree);

                    if !context.event_queue.is_empty() {
                        event_loop_proxy.send_event(Event::new(()));
                    }

                    apply_transform(&mut context, &tree);

                    apply_hover(&mut context);

                    apply_clipping(&mut context, &tree);

                    if let Some(window_view) = context.views.get(&primary_window) {
                        if let Some(window) = window_view.downcast_ref::<Window>() {
                            if context.style.borrow().needs_redraw {
                                window.handle.window().request_redraw();
                                context.style.borrow_mut().needs_redraw = false;
                            }
                        }

                        if let Some(window) = window_view.downcast_ref::<RawWindow>() {
                            if let Some(handle) = &window.window {
                                handle.request_redraw();
                            }
                        }
                    }



                    if let Some(idle_callback) = &on_idle {
                        context.current = primary_window;
                        context.count = 0;
                        (idle_callback)(&mut context);

                        if !context.event_queue.is_empty() {
                            event_loop_proxy.send_event(Event::new(())).unwrap();
                        }
                    }
                }

                glutin::event::Event::RedrawRequested(window_id) => {
                    // Redraw here
                    //println!("Redraw");
                    
                    if let Some(window_entity) = windows.get(&window_id) {
                        if let Some(mut window_view) = context.views.remove(window_entity) {
                            if let Some(window) = window_view.downcast_mut::<Window>() {
                                
                                let window_width = context.cache.get_width(*window_entity);
                                let window_height = context.cache.get_height(*window_entity);
    
                                window.canvas.set_size(window_width as u32, window_height as u32, dpi_factor as f32);
                                let clear_color = context.style.borrow_mut().background_color.get(*window_entity).cloned().unwrap_or(Color::white());
                                window.canvas.clear_rect(
                                    0,
                                    0,
                                    window_width as u32,
                                    window_height as u32,
                                    clear_color.into(),
                                );
    
                                // Sort the tree by z order
                                let mut draw_tree: Vec<Entity> = context.tree.into_iter().collect();
                                draw_tree.sort_by_cached_key(|entity| context.cache.get_z_index(*entity));
    
                                for entity in draw_tree.into_iter() {
    
    
                                    // Skip window
                                    if entity == *window_entity {
                                        continue;
                                    }
    
                                    // Skip invisible widgets
                                    if context.cache.get_visibility(entity) == Visibility::Invisible {
                                        continue;
                                    }
    
                                    if context.cache.get_display(entity) == Display::None {
                                        continue;
                                    }
    
                                    // Skip widgets that have 0 opacity
                                    if context.cache.get_opacity(entity) == 0.0 {
                                        continue;
                                    }
    
                                    // Apply clipping
                                    let clip_region = context.cache.get_clip_region(entity);
                                    window.canvas.scissor(
                                        clip_region.x,
                                        clip_region.y,
                                        clip_region.w,
                                        clip_region.h,
                                    );
                            
                                    // Apply transform
                                    let transform = context.cache.get_transform(entity);
                                    window.canvas.save();
                                    window.canvas.set_transform(transform[0], transform[1], transform[2], transform[3], transform[4], transform[5]);
    
                                    if let Some(view) = context.views.remove(&entity) {
    
                                        context.current = entity;
                                        view.draw(&context, &mut window.canvas);
                                        
                                        context.views.insert(entity, view);
                                    }
    
                                    window.canvas.restore();
                                }
    
                                window.canvas.flush();
                                window.handle.swap_buffers().expect("Failed to swap buffers");
                            }
    
                            context.views.insert(*window_entity, window_view);
                        }

                    }
                    
                    
                }
                
                glutin::event::Event::WindowEvent {
                    window_id,
                    event,
                } => {
                    match event {
                        glutin::event::WindowEvent::CloseRequested => {
                            if let Some(window_entity) = windows.get(&window_id) {
                                context.remove(*window_entity);
                            }
                            windows.remove(&window_id);

                            if windows.is_empty() {
                                *control_flow = ControlFlow::Exit;
                            }
                        }

                        glutin::event::WindowEvent::CursorMoved {
                            device_id: _,
                            position,
                            modifiers: _
                        } => {

                            context.mouse.cursorx = position.x as f32;
                            context.mouse.cursory = position.y as f32;

                            apply_hover(&mut context);

                            if context.captured != Entity::null() {
                                context.event_queue.push_back(
                                    Event::new(WindowEvent::MouseMove(context.mouse.cursorx, context.mouse.cursory))
                                        .target(context.captured)
                                        .propagate(Propagation::Direct),
                                );
                            } else if context.hovered != primary_window {
                                context.event_queue.push_back(
                                    Event::new(WindowEvent::MouseMove(context.mouse.cursorx, context.mouse.cursory))
                                        .target(context.hovered),
                                );
                            }
                        }

                        glutin::event::WindowEvent::MouseInput {
                            device_id: _,
                            button,
                            state,
                            modifiers: _,
                        } => {
                            let button = match button {
                                glutin::event::MouseButton::Left => MouseButton::Left,
                                glutin::event::MouseButton::Right => MouseButton::Right,
                                glutin::event::MouseButton::Middle => MouseButton::Middle,
                                glutin::event::MouseButton::Other(val) => MouseButton::Other(val),
                            };

                            let state = match state {
                                glutin::event::ElementState::Pressed => MouseButtonState::Pressed,
                                glutin::event::ElementState::Released => MouseButtonState::Released,
                            };

                            match state {
                                MouseButtonState::Pressed => {
                                    //context.event_queue.push_back(Event::new(WindowEvent::MouseDown(button)).target(context.hovered).propagate(Propagation::Up));
                                
                                    if context.captured != Entity::null() {
                                        context.event_queue.push_back(
                                            Event::new(WindowEvent::MouseDown(button))
                                                .target(context.captured)
                                                .propagate(Propagation::Direct),
                                        );
                                    } else {
                                        context.event_queue.push_back(
                                            Event::new(WindowEvent::MouseDown(button))
                                                .target(context.hovered),
                                        );
                                    }

                                    match button {
                                        MouseButton::Left => {
                                            context.mouse.left.pos_down =
                                                (context.mouse.cursorx, context.mouse.cursory);
                                                context.mouse.left.pressed = context.hovered;
                                        }

                                        MouseButton::Right => {
                                            context.mouse.right.pos_down =
                                                (context.mouse.cursorx, context.mouse.cursory);
                                                context.mouse.right.pressed = context.hovered;
                                        }

                                        MouseButton::Middle => {
                                            context.mouse.middle.pos_down =
                                                (context.mouse.cursorx, context.mouse.cursory);
                                                context.mouse.middle.pressed = context.hovered;
                                        }

                                        _=> {}
                                    }
                                }

                                MouseButtonState::Released => {
                                    //context.event_queue.push_back(Event::new(WindowEvent::MouseUp(button)).target(context.hovered).propagate(Propagation::Up));
                                
                                    if context.captured != Entity::null() {
                                        context.event_queue.push_back(
                                            Event::new(WindowEvent::MouseUp(button))
                                                .target(context.captured)
                                                .propagate(Propagation::Direct),
                                        );
                                    } else {
                                        context.event_queue.push_back(
                                            Event::new(WindowEvent::MouseUp(button))
                                                .target(context.hovered),
                                        );
                                    }
                                }
                            }
                        }

                        glutin::event::WindowEvent::KeyboardInput {
                            device_id: _,
                            input,
                            is_synthetic: _,
                        } => {
                            if input.virtual_keycode == Some(VirtualKeyCode::H) && input.state == ElementState::Pressed {
                                println!("Tree");
                                for entity in context.tree.into_iter() {
                                    println!("Entity: {} Parent: {:?} posx: {} posy: {} width: {} height: {}", entity, entity.parent(&context.tree), context.cache.get_posx(entity), context.cache.get_posy(entity), context.cache.get_width(entity), context.cache.get_height(entity));
                                }
                            }

                            
                            if input.virtual_keycode == Some(VirtualKeyCode::F5) && input.state == ElementState::Pressed {
                                context.reload_styles().unwrap();
                            }

                            let s = match input.state {
                                glutin::event::ElementState::Pressed => MouseButtonState::Pressed,
                                glutin::event::ElementState::Released => MouseButtonState::Released,
                            };

	                        // Prefer virtual keycodes to scancodes, as scancodes aren't uniform between platforms
	                        let code = if let Some(vkey) = input.virtual_keycode {
		                        vcode_to_code(vkey)
	                        } else {
		                        scan_to_code(input.scancode)
	                        };

                            let key = vk_to_key(
                                input.virtual_keycode.unwrap_or(VirtualKeyCode::NoConvert),
                            );

                            match s {
                                MouseButtonState::Pressed => {
                                    if context.focused != Entity::null() {
                                        context.event_queue.push_back(
                                            Event::new(WindowEvent::KeyDown(code, key))
                                                .target(context.focused)
                                                .propagate(Propagation::Up),
                                        );
                                    } else {
                                        context.event_queue.push_back(
                                            Event::new(WindowEvent::KeyDown(code, key))
                                                .target(context.hovered)
                                                .propagate(Propagation::Up),
                                        );
                                    }
                                }

                                MouseButtonState::Released => {
                                    if context.focused != Entity::null() {
                                        context.event_queue.push_back(
                                            Event::new(WindowEvent::KeyUp(code, key))
                                                .target(context.focused)
                                                .propagate(Propagation::Up),
                                        );
                                    } else {
                                        context.event_queue.push_back(
                                            Event::new(WindowEvent::KeyUp(code, key))
                                                .target(context.hovered)
                                                .propagate(Propagation::Up),
                                        );
                                    }
                                }
                            }
                        }

                        glutin::event::WindowEvent::ReceivedCharacter(character) => {
                            context.event_queue.push_back(
                                Event::new(WindowEvent::CharInput(character))
                                    .target(context.focused)
                                    .propagate(Propagation::Up),
                            );
                        }

                        glutin::event::WindowEvent::Resized(size) => {
                            
                            
                            if let Some(window_entity) = windows.get(&window_id) {
                                //println!("Resized: {:?} {:?}", window_entity, size);
                                if let Some(mut window_view) = context.views.remove(window_entity) {
                                    if let Some(window) = window_view.downcast_mut::<Window>() {
                                    
                                        window.handle.resize(size);
                                    }
                                    
                                    context.views.insert(*window_entity, window_view);

                                }
    
                                context
                                    .style
                                    .borrow_mut()
                                    .width
                                    .insert(*window_entity, Units::Pixels(size.width as f32));
    
                                context
                                    .style
                                    .borrow_mut()
                                    .height
                                    .insert(*window_entity, Units::Pixels(size.height as f32));
    
                                context
                                    .cache
                                    .set_width(*window_entity, size.width as f32);
                                context
                                    .cache
                                    .set_height(*window_entity, size.height as f32);
    
                                let mut bounding_box = BoundingBox::default();
                                bounding_box.w = size.width as f32;
                                bounding_box.h = size.height as f32;
                            
                                context.cache.set_clip_region(*window_entity, bounding_box);
    
                                context.style.borrow_mut().needs_restyle = true;
                                context.style.borrow_mut().needs_relayout = true;
                                context.style.borrow_mut().needs_redraw = true;

                            }
                            
                        }

                        glutin::event::WindowEvent::ModifiersChanged(modifiers_state) => {
                            
                            
                            context.modifiers.set(Modifiers::SHIFT, modifiers_state.shift());
                            context.modifiers.set(Modifiers::ALT, modifiers_state.alt());
                            context.modifiers.set(Modifiers::CTRL, modifiers_state.ctrl());
                            context.modifiers.set(Modifiers::LOGO, modifiers_state.logo());
                            
                        }



                        _=> {}
                    }
                }
                
                _=> {}
            }
        });
    }
}

// fn debug(cx: &mut Context, entity: Entity) -> String {
//     if let Some(view) = cx.views.get(&entity) {
//         view.debug(entity)
//     } else {
//         "None".to_string()
//     }
// }