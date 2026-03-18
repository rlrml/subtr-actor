use crate::*;
use boxcars;
pub use derive_new;
pub use paste;
use std::sync::Arc;

pub trait FeatureAdder<F> {
    fn features_added(&self) -> usize {
        self.get_column_headers().len()
    }

    fn get_column_headers(&self) -> &[&str];

    fn add_features(
        &self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_count: usize,
        current_time: f32,
        vector: &mut Vec<F>,
    ) -> SubtrActorResult<()>;
}

pub type FeatureAdders<F> = Vec<Arc<dyn FeatureAdder<F> + Send + Sync>>;

pub trait LengthCheckedFeatureAdder<F, const N: usize> {
    fn get_column_headers_array(&self) -> &[&str; N];

    fn get_features(
        &self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_count: usize,
        current_time: f32,
    ) -> SubtrActorResult<[F; N]>;
}

#[macro_export]
macro_rules! impl_feature_adder {
    ($struct_name:ident) => {
        impl<F: TryFrom<f32>> FeatureAdder<F> for $struct_name<F>
        where
            <F as TryFrom<f32>>::Error: std::fmt::Debug,
        {
            fn add_features(
                &self,
                processor: &ReplayProcessor,
                frame: &boxcars::Frame,
                frame_count: usize,
                current_time: f32,
                vector: &mut Vec<F>,
            ) -> SubtrActorResult<()> {
                Ok(
                    vector.extend(self.get_features(
                        processor,
                        frame,
                        frame_count,
                        current_time,
                    )?),
                )
            }

            fn get_column_headers(&self) -> &[&str] {
                self.get_column_headers_array()
            }
        }
    };
}

pub trait PlayerFeatureAdder<F> {
    fn features_added(&self) -> usize {
        self.get_column_headers().len()
    }

    fn get_column_headers(&self) -> &[&str];

    fn add_features(
        &self,
        player_id: &PlayerId,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_count: usize,
        current_time: f32,
        vector: &mut Vec<F>,
    ) -> SubtrActorResult<()>;
}

pub type PlayerFeatureAdders<F> = Vec<Arc<dyn PlayerFeatureAdder<F> + Send + Sync>>;

pub trait LengthCheckedPlayerFeatureAdder<F, const N: usize> {
    fn get_column_headers_array(&self) -> &[&str; N];

    fn get_features(
        &self,
        player_id: &PlayerId,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_count: usize,
        current_time: f32,
    ) -> SubtrActorResult<[F; N]>;
}

#[macro_export]
macro_rules! impl_player_feature_adder {
    ($struct_name:ident) => {
        impl<F: TryFrom<f32>> PlayerFeatureAdder<F> for $struct_name<F>
        where
            <F as TryFrom<f32>>::Error: std::fmt::Debug,
        {
            fn add_features(
                &self,
                player_id: &PlayerId,
                processor: &ReplayProcessor,
                frame: &boxcars::Frame,
                frame_count: usize,
                current_time: f32,
                vector: &mut Vec<F>,
            ) -> SubtrActorResult<()> {
                Ok(vector.extend(self.get_features(
                    player_id,
                    processor,
                    frame,
                    frame_count,
                    current_time,
                )?))
            }

            fn get_column_headers(&self) -> &[&str] {
                self.get_column_headers_array()
            }
        }
    };
}

impl<G, F, const N: usize> FeatureAdder<F> for (G, &[&str; N])
where
    G: Fn(&ReplayProcessor, &boxcars::Frame, usize, f32) -> SubtrActorResult<[F; N]>,
{
    fn add_features(
        &self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_count: usize,
        current_time: f32,
        vector: &mut Vec<F>,
    ) -> SubtrActorResult<()> {
        vector.extend(self.0(processor, frame, frame_count, current_time)?);
        Ok(())
    }

    fn get_column_headers(&self) -> &[&str] {
        self.1.as_slice()
    }
}

impl<G, F, const N: usize> PlayerFeatureAdder<F> for (G, &[&str; N])
where
    G: Fn(&PlayerId, &ReplayProcessor, &boxcars::Frame, usize, f32) -> SubtrActorResult<[F; N]>,
{
    fn add_features(
        &self,
        player_id: &PlayerId,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_count: usize,
        current_time: f32,
        vector: &mut Vec<F>,
    ) -> SubtrActorResult<()> {
        vector.extend(self.0(
            player_id,
            processor,
            frame,
            frame_count,
            current_time,
        )?);
        Ok(())
    }

    fn get_column_headers(&self) -> &[&str] {
        self.1.as_slice()
    }
}

#[macro_export]
macro_rules! build_global_feature_adder {
    ($struct_name:ident, $prop_getter:expr, $( $column_names:expr ),* $(,)?) => {

        #[derive(derive_new::new)]
        pub struct $struct_name<F> {
            _zero: std::marker::PhantomData<F>,
        }

        impl<F: Sync + Send + TryFrom<f32> + 'static> $struct_name<F> where
            <F as TryFrom<f32>>::Error: std::fmt::Debug,
        {
            pub fn arc_new() -> std::sync::Arc<dyn FeatureAdder<F> + Send + Sync + 'static> {
                std::sync::Arc::new(Self::new())
            }
        }

        global_feature_adder!(
            $struct_name,
            $prop_getter,
            $( $column_names ),*
        );
    }
}

#[macro_export]
macro_rules! global_feature_adder {
    ($struct_name:ident, $prop_getter:expr, $( $column_names:expr ),* $(,)?) => {
        macro_rules! _global_feature_adder {
            ($count:ident) => {
                impl<F: TryFrom<f32>> LengthCheckedFeatureAdder<F, $count> for $struct_name<F>
                where
                    <F as TryFrom<f32>>::Error: std::fmt::Debug,
                {
                    fn get_column_headers_array(&self) -> &[&str; $count] {
                        &[$( $column_names ),*]
                    }

                    fn get_features(
                        &self,
                        processor: &ReplayProcessor,
                        frame: &boxcars::Frame,
                        frame_count: usize,
                        current_time: f32,
                    ) -> SubtrActorResult<[F; $count]> {
                        $prop_getter(self, processor, frame, frame_count, current_time)
                    }
                }

                impl_feature_adder!($struct_name);
            };
        }
        paste::paste! {
            const [<$struct_name:snake:upper _LENGTH>]: usize = [$($column_names),*].len();
            _global_feature_adder!([<$struct_name:snake:upper _LENGTH>]);
        }
    }
}

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
                        processor: &ReplayProcessor,
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

pub fn convert_float_conversion_error<T>(_: T) -> SubtrActorError {
    SubtrActorError::new(SubtrActorErrorVariant::FloatConversionError)
}

#[macro_export]
macro_rules! convert_all {
    ($err:expr, $( $item:expr ),* $(,)?) => {{
		Ok([
			$( $item.try_into().map_err($err)? ),*
		])
	}};
}

#[macro_export]
macro_rules! convert_all_floats {
    ($( $item:expr ),* $(,)?) => {{
        convert_all!(convert_float_conversion_error, $( $item ),*)
    }};
}
