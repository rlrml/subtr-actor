#[macro_export]
macro_rules! attribute_match {
    ($value:expr, $type:path $(,)?) => {{
        let attribute = $value;
        if let $type(value) = attribute {
            Ok(value)
        } else {
            SubtrActorError::new_result(SubtrActorErrorVariant::UnexpectedAttributeType {
                expected_type: stringify!($type),
                actual_type: attribute_type_name(&attribute),
            })
        }
    }};
}

#[macro_export]
macro_rules! get_attribute_errors_expected {
    ($self:ident, $map:expr, $prop:expr, $type:path) => {
        $self
            .get_attribute($map, $prop)
            .and_then(|found| attribute_match!(found, $type))
    };
}

macro_rules! get_attribute_and_updated {
    ($self:ident, $map:expr, $prop:expr, $type:path) => {
        $self
            .get_attribute_and_updated($map, $prop)
            .and_then(|(found, updated)| attribute_match!(found, $type).map(|v| (v, updated)))
    };
}

macro_rules! get_actor_attribute_matching {
    ($self:ident, $actor:expr, $prop:expr, $type:path) => {
        $self
            .get_actor_attribute($actor, $prop)
            .and_then(|found| attribute_match!(found, $type))
    };
}

macro_rules! get_derived_attribute {
    ($map:expr, $key:expr, $type:path) => {
        $map.get($key)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::DerivedKeyValueNotFound {
                    name: $key.to_string(),
                })
            })
            .and_then(|found| attribute_match!(&found.0, $type))
    };
}
