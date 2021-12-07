use std::{any::{Any, TypeId}, marker::PhantomData};


mod id;
pub use id::*;

mod input;
pub use input::*;

mod localization;
pub use localization::*;

mod entity;
pub use entity::*;

mod handle;
pub use handle::*;

mod tree;
pub use morphorm::*;
pub use style::{Abilities, Color};
pub use tree::*;

pub mod views;
pub use views::*;

mod context;
pub use context::*;

mod window;
pub use window::*;

mod raw_window;
pub use raw_window::*;

mod application;
pub use application::*;

mod events;
pub use events::*;

mod storage;

mod style;
pub use style::{Style, Rule, Display, Visibility, PseudoClass, Overflow};

mod animation;
pub use animation::*;

mod data;
pub use data::*;

mod layout;
pub use layout::*;

mod resource;
pub use resource::*;

mod mouse;
pub use mouse::*;

mod binding;
pub use binding::*;

mod hover_system;
pub use hover_system::apply_hover;

mod style_system;
pub use style_system::apply_styles;

pub use morphorm::Units::*;

pub use vizia_derive::{Data, Lens};

mod view;
pub use view::{View, Canvas};

mod extention;
pub use extention::*;

pub use keyboard_types::{Code, Key};

// pub trait Model: Sized {
//     fn build(&self, cx: &mut Context) -> TypedId<Self>;
// }

// pub struct Wrapper<D> {
//     id: TypedId<D>,
// }

// impl<D> Wrapper<D> {
//     pub fn build<F>(&mut self, cx: &mut Context, f: F)
//     where F: 'static + Fn(&mut Context, &D) {
//         // Add widget to context
//         // Get data from context
//         // Pass data to build closure 
//         if let Some(data) = cx.data.get(&self.id.id) {
//             // Downcast data to correct type
//             // Pass data to build closure
//             (f)(cx, data)
//         }
//     }
// }

// #[derive(Clone, Copy)]
// pub struct TypedId<T: Sized> {
//     id: u32,
//     p: PhantomData<T>,
// }

// pub trait View: Sized {
//     fn body(&mut self, cx: &mut Context) {}
//     fn build(mut self, cx: &mut Context) {
//         let id = cx.entity_manager.create();
//         cx.tree.add(id, cx.current);
//         cx.cache.add(id);
//         cx.current = id;
//         self.body(cx);
//     }
// }




// #[derive(Clone, Copy)]
// pub struct State<T> {
//     id: StateID,

//     p: PhantomData<T>,
// }

// impl<T: StateTrait> State<T> {
//     pub fn get<'a>(&self, cx: &'a Context) -> &'a T {
//         cx.state.get(&self.id).unwrap().downcast_ref::<T>().unwrap()
//     }

//     pub fn set<F>(&self, cx: &mut Context, f: F) 
//     where F: FnOnce(&mut T)
//     {
//         //println!("Set Value");
//         // Tell context that the state has changed
//         // This will rebuild the view attached the the state
//         // and then replace the state with the new value
//         //let current = cx.state.get(&self.id).unwrap().downcast_ref::<T>().unwrap();
//         //if current != &val {
//             let val = cx.state.get_mut(&self.id).unwrap().downcast::<T>().unwrap();
//             (f)(val);
        
//             // for child in self.id.view.child_iter(&cx.tree.clone()) {
//             //     cx.remove(child);
//             // }
//             if let Some(mut view) = cx.views.remove(&self.id.view) {
//                 let prev = cx.current;
//                 cx.current = self.id.view;
//                 cx.state_count = 0;
//                 cx.count = 0;
//                 view.body(cx);
//                 cx.current = prev;
    
//                 cx.views.insert(self.id.view, view);

//                 morphorm::layout(&mut cx.cache, &cx.tree, &cx.style.clone().borrow());
//                 apply_hover(cx);
//             } else {
//                 println!("No Builder: {}", self.id.view);
//             }
//         //}
        
//     } 
// }

// pub trait StateTrait: 'static + Sized + PartialEq {
//     fn build(self, cx: &mut Context) -> State<Self> {
//         //let id = cx.entity_manager.create();
//         //println!("{} {}", cx.current, cx.state_count);
//         let id = StateID {
//             view: cx.current,
//             index: cx.state_count,
//         };
//         if !cx.state.contains_key(&id) {
//             cx.state.insert(id, Box::new(self));
//         }
        
//         cx.state_count += 1;
        
//         State {
//             id,
//             p: Default::default(),
//         }
//     }
// }

// impl StateTrait for String {

// }

// impl StateTrait for i32 {

// }

// #[derive(Hash, PartialEq, Eq, Clone, Copy)]
// pub struct StateID {
//     view: Entity,
//     index: u32,
// }

// /// Extension on the `Any` trait which provides downcasting methods.
// pub trait StateData: Any {

// }

// impl dyn StateData {
//     // Check if a message is a certain type
//     pub fn is<T: Any + 'static>(&self) -> bool {
//         // Get TypeId of the type this function is instantiated with
//         let t = TypeId::of::<T>();

//         // Get TypeId of the type in the trait object
//         let concrete = self.type_id();

//         // Compare both TypeIds on equality
//         t == concrete
//     }

//     // Casts a message to the specified type if the message is of that type
//     pub fn downcast<T>(&mut self) -> Option<&mut T>
//     where
//         T: StateData + 'static,
//     {
//         if self.is::<T>() {
//             unsafe { Some(&mut *(self as *mut dyn StateData as *mut T)) }
//         } else {
//             None
//         }
//     }

//     pub fn downcast_ref<T>(&self) -> Option<&T>
//     where
//         T: Any + 'static,
//     {
//         if self.is::<T>() {
//             unsafe { Some(&*(self as *const dyn StateData as *const T)) }
//         } else {
//             None
//         }
//     }
// }

// trait Downcast {
//     fn as_any (self: &'_ Self)
//       -> &'_ dyn Any
//     where
//         Self : 'static,
//     ;
// }

// impl<T: StateData> Downcast for T {
//     fn as_any (self: &'_ Self)
//       -> &'_ dyn Any
//     where
//         Self : 'static,
//     {
//         self
//     }
// }

// impl<T: StateTrait> StateData for T {

// }

// impl<T: Container> Stylable for T {
//     type Ret = StyleBuilder<Self,C>;
//     fn background_color(self, color: Color) -> StyleBuilder<Self,C> {
//         StyleBuilder::new(self).background_color(color)
//     }
// }

// impl<T: View> Stylable for T {
//     type Ret = StyleBuilder<Self,N>;
//     fn background_color(self, color: Color) -> StyleBuilder<Self,C> {
//         StyleBuilder::new(self).background_color(color)
//     }
// }