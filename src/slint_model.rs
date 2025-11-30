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
                    [<$field_name _sender>],
                    $field_vis $field_name: $field_type
                ),*
            }
        } }
    };

    (@inner
        $vis: vis $model_name:ident: $type_name:ident<$component_type:ty> {
            $(
                $sender_name:ident,
                $field_vis: vis $field_name:ident: $field_type:ty
            ),*
        }
    ) => {
        $vis struct $model_name {
            $( $field_vis $field_name: slint_model!{ @def_field $component_type, $field_type, } ),*
        }

        impl $model_name {
            pub fn new(
                component: &$component_type,
            ) -> Self
            where $component_type: slint::ComponentHandle {
                let weak = std::sync::Arc::new(component.as_weak());

                $(
                    let $field_name = slint_model!{ @new_field
                        $component_type,
                        $type_name,
                        $field_name,
                        $field_type,
                        weak,
                    };
                )*

                $( let $sender_name = $field_name.sender().clone(); )*

                $(
                    slint_model!{ @bind_field
                        $type_name,
                        $sender_name,
                        $field_name,
                        $field_type,
                        component,
                    };
                )*

                Self {
                    $( $field_name ),*
                }
            }
        }
    };

    (@def_field
        $component_type:ty, (),
    ) => (
        Property<slint::Weak<$component_type>, ()>
    );

    (@def_field
        $component_type:ty, $field_type:ty,
    ) => (
        Property<slint::Weak<$component_type>, $field_type>
    );

    (@set_field_name
        $field_name:ident,
        (),
    ) => {

    };

    (@set_field_name
        $field_name:ident,
        $field_type:ty,
    ) => {
        paste::paste! { [<set_ $field_name>] }
    };

    (@new_field
        $component_type:ty,
        $type_name:ident,
        $field_name:ident,
        (),
        $weak:ident,
    ) => (
        Property::new(
            $weak.clone(),
            (),
            |_, _| {},
        )
    );

    (@new_field
        $component_type:ty,
        $type_name:ident,
        $field_name:ident,
        $field_type:ty,
        $weak:ident,
    ) => (
        paste::paste! {
            Property::new(
                $weak.clone(),
                Default::default(),
                |c, v| {
                    c.upgrade_in_event_loop(move |c| {
                        c.global::<$type_name>().[<set_ $field_name>](v)
                    }).unwrap() // TODO: Error handling
                },
            )
        }
    );

    (@bind_field
        $type_name:ident,
        $sender_name:ident,
        $field_name:ident,
        (),
        $component:ident,
    ) => (
        paste::paste! {
            $component.global::<$type_name>().[<on_ $field_name>](move || $sender_name.send(()))
        }
    );

    (@bind_field
        $type_name:ident,
        $sender_name:ident,
        $field_name:ident,
        $field_type:ty,
        $component:ident,
    ) => (
        paste::paste! {
            $component.global::<$type_name>().[<on_changed_ $field_name>](move |v| $sender_name.send(v))
        }
    );
}

