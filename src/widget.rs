use bevy::{
    ecs::system::SystemParam,
    prelude::{Changed, Component, Entity, In, Query, With},
};

use crate::{
    children::KChildren,
    context::{Mounted, WidgetName},
    prelude::WidgetContext,
    styles::KStyle,
};

pub trait Widget: Send + Sync {
    fn get_name(&self) -> WidgetName {
        WidgetName(std::any::type_name::<Self>().into())
    }
}

#[derive(Component, Default, PartialEq, Clone)]
pub struct EmptyState;

/// Used to diff widget props.
pub trait WidgetProps {}

pub fn widget_update<
    Props: WidgetProps + PartialEq + Component + Clone,
    State: PartialEq + Component + Clone,
>(
    In((widget_context, entity, previous_entity)): In<(WidgetContext, Entity, Entity)>,
    widget_param: WidgetParam<Props, State>,
) -> bool {
    widget_param.has_changed(&widget_context, entity, previous_entity)
}

pub fn widget_update_with_context<
    Props: WidgetProps + PartialEq + Component + Clone,
    State: PartialEq + Component + Clone,
    Context: PartialEq + Component + Clone + Default,
>(
    In((widget_context, entity, previous_entity)): In<(WidgetContext, Entity, Entity)>,
    widget_param: WidgetParam<Props, State>,
    context_query: Query<Entity, Changed<Context>>,
) -> bool {
    // Uses bevy state changes to see if context has changed.
    if let Some(context_entity) = widget_context.get_context_entity::<Context>(entity) {
        if context_query.contains(context_entity) {
            return true;
        }
    }

    widget_param.has_changed(&widget_context, entity, previous_entity)
}

#[derive(SystemParam)]
pub struct WidgetParam<
    'w,
    's,
    Props: WidgetProps + PartialEq + Component,
    State: PartialEq + Component,
> {
    pub props_query: Query<'w, 's, &'static Props>,
    pub old_props_query: Query<'w, 's, &'static Props>,
    pub mounted_query: Query<'w, 's, Entity, With<Mounted>>,
    pub style_query: Query<'w, 's, &'static KStyle>,
    pub children_query: Query<'w, 's, &'static KChildren>,
    pub state_query: Query<'w, 's, &'static State>,
    pub widget_names: Query<'w, 's, &'static WidgetName>,
}

impl<'w, 's, Props: WidgetProps + PartialEq + Component, State: PartialEq + Component>
    WidgetParam<'w, 's, Props, State>
{
    pub fn has_changed(
        &self,
        widget_context: &WidgetContext,
        current_entity: Entity,
        previous_entity: Entity,
    ) -> bool {
        if !self.mounted_query.is_empty() {
            return true;
        }

        // Compare styles
        if let (Ok(style), Ok(old_style)) = (
            self.style_query.get(current_entity),
            self.style_query.get(previous_entity),
        ) {
            if style != old_style {
                log::trace!(
                    "Entity styles have changed! {}-{}",
                    self.widget_names.get(current_entity).unwrap().0,
                    current_entity.id()
                );
                return true;
            }
        }

        // Compare children
        // If children don't exist ignore as mount will add them!
        if let (Ok(children), Ok(old_children)) = (
            self.children_query.get(current_entity),
            self.children_query.get(previous_entity),
        ) {
            if children != old_children {
                return true;
            }
        }

        // Check props
        if let (Ok(props), Ok(previous_props)) = (
            self.props_query.get(current_entity),
            self.old_props_query.get(previous_entity),
        ) {
            if previous_props != props {
                log::trace!(
                    "Entity props have changed! {}-{}",
                    self.widget_names.get(current_entity).unwrap().0,
                    current_entity.id()
                );
                return true;
            }
        }

        // Check state
        let previous_state_entity = widget_context.get_state(previous_entity);
        let current_state_entity = widget_context.get_state(current_entity);

        // Check if state was nothing but now is something
        if current_state_entity.is_some() && previous_state_entity.is_none() {
            return true;
        }

        // Check state
        if current_state_entity.is_some() && previous_state_entity.is_some() {
            let previous_state_entity = previous_state_entity.unwrap();
            let current_state_entity = current_state_entity.unwrap();
            if let (Ok(state), Ok(previous_state)) = (
                self.state_query.get(current_state_entity),
                self.state_query.get(previous_state_entity),
            ) {
                if previous_state != state {
                    return true;
                }
            }
        }

        false
    }
}
