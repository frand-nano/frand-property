#[cfg(feature = "slint")]
#[macro_export]
macro_rules! slint_model {
    (
        $vis: vis $model_name:ident: $type_name:ident<$component_type:ty> {
            $( $field_vis: vis $field_name:ident : $field_type:ty ),* $(,)?
        }
    ) => {
        paste::paste! { slint_model! {
            @inner
            $vis $model_name: $type_name<$component_type> {
                $(
                    [<set_ $field_name>],
                    [<on_changed_ $field_name>],
                    $field_name: $field_type
                ),*
            }
        } }
    };

    (@inner
        $vis: vis $model_name:ident: $type_name:ident<$component_type:ty> {
            $(
                $setter_name:ident,
                $callback_name:ident,
                $field_vis: vis $field_name:ident: $field_type:ty
            ),*
        }
    ) => {
        $vis struct $model_name {
            $( $field_vis $field_name: slint_model!{ @def_field $component_type, $field_type, } ),*
        }

        impl $model_name {
            pub fn new(
                component: &slint::Weak<$component_type>,
            ) -> Self
            where $component_type: slint::ComponentHandle {
                Self {
                    $( $field_name: slint_model!{ @new_field $type_name, $setter_name, $callback_name, $field_type, component, } ),*
                }
            }
        }
    };

    (@def_field
        $component_type:ty, (),
    ) => (
        Event<slint::Weak<$component_type>>
    );

    (@def_field
        $component_type:ty, $field_type:ty,
    ) => (
        Property<slint::Weak<$component_type>, $field_type>
    );

    (@new_field
        $type_name:ident,
        $setter_name:ident,
        $callback_name:ident,
        (),
        $component:ident,
    ) => (
        Event::new(
            $component.clone(),
        )
    );

    (@new_field
        $type_name:ident,
        $setter_name:ident,
        $callback_name:ident,
        $field_type:ty,
        $component:ident,
    ) => (
        Property::new(
            $component.clone(),
            Default::default(),
            |c, v| {
                c.upgrade_in_event_loop(move |c| {
                    c.global::<$type_name>().$setter_name(v)
                }).unwrap() // TODO: Error handling
            },
        )
    );
}

