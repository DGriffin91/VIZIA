use std::{any::TypeId, cell::RefCell, collections::{HashMap, VecDeque}, io::Read, rc::Rc};

use femtovg::FontId;
use fluent_bundle::{FluentBundle, FluentResource};
use unic_langid::LanguageIdentifier;

use crate::{AppData, CachedData, Data, Entity, Event, FontOrId, IdManager, Lens, LensWrap, Message, Modifiers, MouseState, Propagation, ResourceManager, Store, Style, Tree, TreeExt, ViewHandler};

pub struct Enviroment {
    // Signifies whether the app should be rebuilt
    // Changing an enviroment variable requires a rebuild of the app
    pub needs_rebuild: bool,
    //pub bundle: FluentBundle<FluentResource>,
}

impl Enviroment {
    pub fn new() -> Self {
        // let lang =  "en-US".parse::<LanguageIdentifier>().expect("Failed to parse locale");
        // let resolved_locales = vec![&lang];
        // let mut bundle = FluentBundle::new(resolved_locales.into_iter().cloned().collect());
        // let mut file = std::fs::File::open("examples/resources/en-US/hello.ftl").expect("No File Found");
        // let mut source: String = String::new();
        // file.read_to_string(&mut source).expect("Failed to read ftl file");
        // let resource = FluentResource::try_new(source).expect("Could not parse an FTL string.");
        // bundle
        //     .add_resource(resource)
        //     .expect("Failed to add FTL resources to the bundle.");
        Self {
            needs_rebuild: true,
            //bundle,
        }
    }

    pub fn set_locale(&mut self, locale: &str) {
        // TODO
        // let lang =  locale.parse::<LanguageIdentifier>().expect("Failed to parse locale");
        // let resolved_locales = vec![&lang];
        // let mut bundle = FluentBundle::new(resolved_locales.into_iter().cloned().collect());
        // let mut file = std::fs::File::open(&format!("examples/resources/{}/hello.ftl", locale)).expect("No File Found");
        // let mut source: String = String::new();
        // file.read_to_string(&mut source).expect("Failed to read ftl file");
        // let resource = FluentResource::try_new(source).expect("Could not parse an FTL string.");
        // bundle
        //     .add_resource(resource)
        //     .expect("Failed to add FTL resources to the bundle.");
        // self.bundle = bundle;
        // self.needs_rebuild = true;
    }
}

pub struct Context {
    pub entity_manager: IdManager<Entity>,
    pub tree: Tree,
    pub current: Entity,
    pub count: usize,
    pub views: HashMap<Entity, Box<dyn ViewHandler>>,
    pub lenses: HashMap<TypeId, Box<dyn LensWrap>>,
    pub data: AppData,
    pub event_queue: VecDeque<Event>,
    pub style: Rc<RefCell<Style>>,
    pub cache: CachedData,

    pub enviroment: Enviroment,

    pub mouse: MouseState,
    pub modifiers: Modifiers,

    pub captured: Entity,
    pub hovered: Entity,
    pub focused: Entity,

    // pub state_count: u32,

    pub resource_manager: ResourceManager,

    // Temp
    pub fonts: Vec<FontId>,
}

impl Context {
    pub fn remove(&mut self, entity: Entity) {
        let delete_list = entity.branch_iter(&self.tree).collect::<Vec<_>>();

        for entity in delete_list.iter().rev() {

            // Remove from observers
            for entry in self.data.model_data.dense.iter_mut() {
                let model_list = &mut entry.value;
                for (_, model) in model_list.iter_mut() {
                    model.remove_observer(*entity);
                }
            }

            //println!("Removing: {}", entity);
            self.tree.remove(*entity).expect("");
            self.cache.remove(*entity);
            self.style.borrow_mut().remove(*entity);
            self.data.model_data.remove(*entity);
            self.entity_manager.destroy(*entity);
            self.views.remove(entity);


        }
    }

    /// Get stored data from the context.
    pub fn data<T: 'static>(&self) -> Option<&T> {
        for entity in self.current.parent_iter(&self.tree) {
            //println!("Current: {} {:?}", entity, entity.parent(&self.tree));
            if let Some(data_list) = self.data.model_data.get(entity) {
                for (_, model) in data_list.iter() {
                    if let Some(store) = model.downcast_ref::<Store<T>>() {
                        return Some(&store.data);
                    }
                }
            }         
        }

        None

    }

    /// Get stored data from the context.
    pub fn data_mut<T: 'static>(&mut self) -> Option<&mut T> {

        let mut found_entity = None;

        for entity in self.current.parent_iter(&self.tree) {
            //println!("Current: {} {:?}", entity, entity.parent(&self.tree));
            if let Some(data_list) = self.data.model_data.get(entity) {
                for (_, model) in data_list.iter() {
                    if let Some(store) = model.downcast_ref::<Store<T>>() {
                        //return Some(&store.data);
                        found_entity = Some(entity);
                        break;
                    }
                }
            }         
        }

        if let Some(entity) = found_entity {
            //println!("Current: {} {:?}", entity, entity.parent(&self.tree));
            if let Some(data_list) = self.data.model_data.get_mut(entity) {
                for (_, model) in data_list.iter_mut() {
                    if let Some(store) = model.downcast::<Store<T>>() {
                        return Some(&mut store.data);
                    }
                }
            }         
        }

        None

    }

    pub fn emit<M: Message>(&mut self, message: M) {
        self.event_queue.push_back(Event::new(message).target(self.current).origin(self.current).propagate(Propagation::Up));
    }

    pub fn emit_trace<M: Message>(&mut self, message: M) {
        self.event_queue.push_back(Event::new(message).target(self.current).origin(self.current).propagate(Propagation::Up).trace());
    }

    /// Add a font from memory to the application
    pub fn add_font_mem(&mut self, name: &str, data: &[u8]) {
        // TODO - return error
        if self.resource_manager.fonts.contains_key(name) {
            println!("Font already exists");
            return;
        }
        //let id = self.text_context.add_font_mem(&data.clone()).expect("failed");
        //println!("{} {:?}", name, id);
        self.resource_manager.fonts.insert(name.to_owned(), FontOrId::Font(data.to_vec()));
    }

    /// Sets the global default font for the application
    pub fn set_default_font(&mut self, name: &str) {
        self.style.borrow_mut().default_font = name.to_string();
    }

    pub fn add_theme(&mut self, theme: &str) {
        self.resource_manager.themes.push(theme.to_owned());

        self.reload_styles().expect("Failed to reload styles");
    }

    pub fn add_stylesheet(&mut self, path: &str) -> Result<(), std::io::Error> {
        let style_string = std::fs::read_to_string(path.clone())?;
        self.resource_manager.stylesheets.push(path.to_owned());
        self.style.borrow_mut().parse_theme(&style_string);

        Ok(())
    }

    pub fn reload_styles(&mut self) -> Result<(), std::io::Error> {
        if self.resource_manager.themes.is_empty() && self.resource_manager.stylesheets.is_empty() {
            return Ok(());
        }

        self.style.borrow_mut().remove_rules();

        self.style.borrow_mut().rules.clear();
        
        self.style.borrow_mut().remove_all();

        let mut overall_theme = String::new();

        // Reload the stored themes
        for theme in self.resource_manager.themes.iter() {
            //self.style.parse_theme(theme);
            overall_theme += theme;
        }

        // Reload the stored stylesheets
        for stylesheet in self.resource_manager.stylesheets.iter() {
            let theme = std::fs::read_to_string(stylesheet)?;
            overall_theme += &theme;
        }

        self.style.borrow_mut().parse_theme(&overall_theme);

        //self.enviroment.needs_rebuild = true;

        // Entity::root().restyle(self);
        // Entity::root().relayout(self);
        // Entity::root().redraw(self);

        Ok(())
    }
}
