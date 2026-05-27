use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(crate) fn module_typed_array<T>(
        &self,
        module_name: &str,
        field: &str,
    ) -> SubtrActorResult<Vec<T>>
    where
        T: DeserializeOwned,
    {
        decode_json_value(Value::Array(self.module_array(module_name, field)))
    }

    pub(crate) fn module_player_events<T, F>(
        &self,
        module_name: &str,
        field: &str,
        parse: F,
    ) -> SubtrActorResult<Vec<T>>
    where
        F: Fn(&Value) -> SubtrActorResult<T>,
    {
        self.module_array(module_name, field)
            .iter()
            .map(parse)
            .collect()
    }

    pub(crate) fn module_array(&self, module_name: &str, field: &str) -> Vec<Value> {
        self.modules
            .get(module_name)
            .and_then(Value::as_object)
            .and_then(|module| module.get(field))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    }
}
