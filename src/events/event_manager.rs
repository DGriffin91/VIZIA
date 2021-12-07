

// use crate::{BindEvent, Context, Display, Entity, Event, FontOrId, Propagation, State, Tree, TreeExt, Visibility, WindowEvent, entity};


use crate::{Context, Event, Propagation, Tree, TreeExt, AppEvent};


/// Dispatches events to widgets.
/// 
/// The [EventManager] is responsible for taking the events in the event queue in state
/// and dispatching them to widgets based on the target and propagation metadata of the event.
/// The is struct is used internally by the application and should not be constructed directly.
pub struct EventManager {

    // Queue of events to be processed
    event_queue: Vec<Event>,

    // A copy of the tree for iteration
    tree: Tree,
}

impl EventManager {
    pub fn new() -> Self {
        EventManager {
            event_queue: Vec::new(),

            tree: Tree::new(),
        }
    }

    pub fn flush_events(&mut self, context: &mut Context) -> Vec<Event> {

        // Clear the event queue in the event manager
        self.event_queue.clear();

        //state.removed_entities.clear();

        // Move events from state to event manager
        self.event_queue.extend(context.event_queue.drain(0..));

        // Sort the events by order
        //self.event_queue.sort_by_cached_key(|event| event.order);

        self.tree = context.tree.clone();

        let mut app_events = Vec::new();

        // Loop over the events in the event queue
        'events: for event in self.event_queue.iter_mut() {

            if let Some(app_event) = event.message.downcast::<AppEvent>() {
                app_events.push(Event::new(*app_event).target(event.target).origin(event.origin));
            }

            if event.trace {
                println!("Event: {:?}", event);
            }

            // Define the target to prevent multiple mutable borrows error
            let target = event.target;

            // Send event to target
            if let Some(mut view) = context.views.remove(&event.target) {
                context.current = event.target;
                view.event(context, event);

                context.views.insert(event.target, view);    
            }

            if let Some(mut model_list) = context.data.model_data.remove(event.target) {
                for (_, model) in model_list.iter_mut() {
                    context.current = event.target;
                    model.event(context, event);
                }

                context.data.model_data.insert(event.target, model_list).expect("Failed to insert data");
            }
            
            if event.consumed {
                continue 'events;
            }

            // if event.trace {
            //     println!("Target: {} Parents: {:?} Tree: {:?}", target, target.parent_iter(&self.tree).collect::<Vec<_>>(), self.tree.parent);
            // }

            // Propagate up from target to root (not including target)
            
            if event.propagation == Propagation::Up {
                // Walk up the tree from parent to parent
                for entity in target.parent_iter(&self.tree) {
                    //println!("Up: {:?} {:?}", entity, self.tree);
                    // Skip the target entity
                    if entity == event.target {
                        continue;
                    }
                    
                    // Send event to all entities before the target
                    if let Some(mut view) = context.views.remove(&entity) {
                        let prev = context.current;
                        context.current = entity;
                        view.event(context, event);
                        context.current = prev;

                        
                        context.views.insert(entity, view);
                    }
                    
                    if let Some(mut model_list) = context.data.model_data.remove(entity) {
                        for (_, model) in model_list.iter_mut() {

                            // if event.trace {
                            //     println!("Event: {:?} -> Model {:?}", event, ty);
                            // }
                            context.current = entity;
                            model.event(context, event);
                        }
    
                        context.data.model_data.insert(entity, model_list).expect("Failed to insert data");
                    }
                    
                    // Skip to the next event if the current event is consumed
                    if event.consumed {
                        continue 'events;
                    }
                }
            }
        }

        app_events
    }
}
