macro_rules! impl_analysis_node {
    (
        node = $node:ident,
        state = $state:ty,
        name = $name:literal,
        dependencies = [$($dependency:expr => $dependency_ty:ty),* $(,)?],
        $(on_replay_meta = |$meta_self:ident, $meta:ident| $on_replay_meta:block,)?
        call = $field:ident.$method:ident
        $(, finish = $finish_field:ident.$finish_method:ident)?
        $(,)?
    ) => {
        impl_analysis_node_call! {
            node = $node,
            state = $state,
            name = $name,
            dependencies = [$($dependency => $dependency_ty),*],
            $(on_replay_meta = |$meta_self, $meta| $on_replay_meta,)?
            call = $field.$method
            $(, finish = $finish_field.$finish_method)?
        }
    };

    (
        node = $node:ident,
        state = $state:ty,
        name = $name:literal,
        dependencies = [$($dependency:expr => $dependency_ty:ty),* $(,)?],
        $(on_replay_meta = |$meta_self:ident, $meta:ident| $on_replay_meta:block,)?
        update_state = $field:ident.$method:ident
        $(, finish = $finish_field:ident.$finish_method:ident)?
        $(,)?
    ) => {
        impl_analysis_node_update_state! {
            node = $node,
            state = $state,
            name = $name,
            dependencies = [$($dependency => $dependency_ty),*],
            $(on_replay_meta = |$meta_self, $meta| $on_replay_meta,)?
            update_state = $field.$method
            $(, finish = $finish_field.$finish_method)?
        }
    };

    (
        node = $node:ident,
        state = $state:ty,
        name = $name:literal,
        dependencies = [$($dependency:expr),* $(,)?],
        inputs = {$($binding:ident : $binding_ty:ty),* $(,)?},
        $(on_replay_meta = |$meta_self:ident, $meta:ident| $on_replay_meta:block,)?
        evaluate = |$eval_self:ident| $evaluate:block,
        $(finish = |$finish_self:ident| $finish:block,)?
        state_ref = |$state_self:ident| $state_ref:expr $(,)?
    ) => {
        impl_analysis_node_custom! {
            node = $node,
            state = $state,
            name = $name,
            dependencies = [$($dependency),*],
            inputs = {$($binding : $binding_ty),*},
            $(on_replay_meta = |$meta_self, $meta| $on_replay_meta,)?
            evaluate = |$eval_self| $evaluate,
            $(finish = |$finish_self| $finish,)?
            state_ref = |$state_self| $state_ref
        }
    };
}
