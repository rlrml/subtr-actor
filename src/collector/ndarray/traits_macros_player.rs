/// Declares a new per-player feature-adder type and wires it into the ndarray traits.
#[macro_export]
macro_rules! build_player_feature_adder {
    ($struct_name:ident, $prop_getter:expr, $( $column_names:expr ),* $(,)?) => {
        #[derive(derive_new::new)]
        pub struct $struct_name<F> {
            _zero: std::marker::PhantomData<F>,
        }

        impl<F: Sync + Send + TryFrom<f32> + 'static> $struct_name<F> where
            <F as TryFrom<f32>>::Error: std::fmt::Debug,
        {
            pub fn arc_new() -> std::sync::Arc<dyn PlayerFeatureAdder<F> + Send + Sync + 'static> {
                std::sync::Arc::new(Self::new())
            }
        }

        player_feature_adder!(
            $struct_name,
            $prop_getter,
            $( $column_names ),*
        );
    }
}

/// Implements the ndarray feature-adder traits for an existing per-player feature type.
#[macro_export]
macro_rules! player_feature_adder {
    ($struct_name:ident, $prop_getter:expr, $( $column_names:expr ),* $(,)?) => {
        macro_rules! _player_feature_adder {
            ($count:ident) => {
                impl<F: TryFrom<f32>> LengthCheckedPlayerFeatureAdder<F, $count> for $struct_name<F>
                where
                    <F as TryFrom<f32>>::Error: std::fmt::Debug,
                {
                    fn get_column_headers_array(&self) -> &[&str; $count] {
                        &[$( $column_names ),*]
                    }

                    fn get_features(
                        &self,
                        player_id: &PlayerId,
                        processor: &dyn ProcessorView,
                        frame: &boxcars::Frame,
                        frame_count: usize,
                        current_time: f32,
                    ) -> SubtrActorResult<[F; $count]> {
                        $prop_getter(self, player_id, processor, frame, frame_count, current_time)
                    }
                }

                impl_player_feature_adder!($struct_name);
            };
        }
        paste::paste! {
            const [<$struct_name:snake:upper _LENGTH>]: usize = [$($column_names),*].len();
            _player_feature_adder!([<$struct_name:snake:upper _LENGTH>]);
        }
    }
}
