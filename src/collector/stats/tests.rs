use serde::{Serialize, Serializer};
use serde_json::json;
use std::sync::Arc;

use crate::{ReplayMeta, StatsReducer, SubtrActorErrorVariant};

use super::{resolve_stats_module_factories, CollectedStats, StatsModule, StatsModuleFactory};

#[derive(Serialize)]
struct FakeModuleData {
    value: &'static str,
}

struct FakeModule {
    name: &'static str,
}

impl StatsReducer for FakeModule {}

impl StatsModule for FakeModule {
    fn name(&self) -> &'static str {
        self.name
    }
}

impl Serialize for FakeModule {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        FakeModuleData { value: self.name }.serialize(serializer)
    }
}

struct FakeFactory {
    key: &'static str,
    name: &'static str,
    dependencies: Vec<Arc<dyn StatsModuleFactory>>,
}

impl StatsModuleFactory for FakeFactory {
    fn key(&self) -> String {
        self.key.to_owned()
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn dependencies(&self) -> Vec<Arc<dyn StatsModuleFactory>> {
        self.dependencies.clone()
    }

    fn build(&self) -> Box<dyn StatsModule> {
        Box::new(FakeModule { name: self.name })
    }
}

fn fake_factory(
    key: &'static str,
    name: &'static str,
    dependencies: Vec<Arc<dyn StatsModuleFactory>>,
) -> Arc<dyn StatsModuleFactory> {
    Arc::new(FakeFactory {
        key,
        name,
        dependencies,
    })
}

#[test]
fn resolver_dedupes_shared_dependencies_and_topologically_orders_them() {
    let shared = fake_factory("shared", "shared", Vec::new());
    let left = fake_factory("left", "left", vec![shared.clone()]);
    let right = fake_factory("right", "right", vec![shared]);

    let resolved =
        resolve_stats_module_factories(vec![left, right]).expect("resolution should succeed");

    assert_eq!(resolved.len(), 3);
    assert_eq!(resolved[0].key, "shared");
    assert_eq!(resolved[1].key, "left");
    assert_eq!(resolved[2].key, "right");
    assert!(!resolved[0].emit);
    assert!(resolved[1].emit);
    assert!(resolved[2].emit);
}

#[test]
fn resolver_rejects_dependency_cycles() {
    let cycle = fake_factory(
        "cycle",
        "cycle",
        vec![fake_factory("cycle", "cycle", Vec::new())],
    );

    let resolution = resolve_stats_module_factories(vec![cycle]);
    assert!(resolution.is_err(), "cycle should fail");
    let error = resolution.err().unwrap();
    assert!(matches!(
        error.variant,
        SubtrActorErrorVariant::StatsModuleDependencyCycle { .. }
    ));
}

#[test]
fn resolver_rejects_duplicate_emitted_module_names() {
    let first = fake_factory("first", "shared_name", Vec::new());
    let second = fake_factory("second", "shared_name", Vec::new());

    let resolution = resolve_stats_module_factories(vec![first, second]);
    assert!(resolution.is_err(), "duplicate names should fail");
    let error = resolution.err().unwrap();
    assert!(matches!(
        error.variant,
        SubtrActorErrorVariant::DuplicateStatsModuleName { .. }
    ));
}

#[test]
fn collected_stats_serialize_modules_by_name() {
    let collected = CollectedStats {
        replay_meta: ReplayMeta {
            team_zero: Vec::new(),
            team_one: Vec::new(),
            all_headers: Vec::new(),
        },
        modules: vec![super::types::CollectedStatsModule {
            name: "fake",
            value: json!({ "value": "fake" }),
        }],
    };

    let value = serde_json::to_value(&collected).expect("serialization should succeed");
    assert_eq!(value["modules"]["fake"]["value"], "fake");
}
