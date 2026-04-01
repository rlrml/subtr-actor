use std::collections::HashMap;
use std::sync::Arc;

use crate::*;

use super::types::StatsModuleFactory;

#[derive(Clone)]
pub(crate) struct ResolvedStatsModuleFactory {
    pub(crate) key: String,
    pub(crate) name: &'static str,
    pub(crate) emit: bool,
    pub(crate) factory: Arc<dyn StatsModuleFactory>,
}

pub(crate) fn resolve_stats_module_factories<I>(
    modules: I,
) -> SubtrActorResult<Vec<ResolvedStatsModuleFactory>>
where
    I: IntoIterator<Item = Arc<dyn StatsModuleFactory>>,
{
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum VisitState {
        Visiting,
        Visited,
    }

    fn visit(
        factory: Arc<dyn StatsModuleFactory>,
        emit: bool,
        visit_states: &mut HashMap<String, VisitState>,
        resolved_indexes: &mut HashMap<String, usize>,
        resolved: &mut Vec<ResolvedStatsModuleFactory>,
        traversal_stack: &mut Vec<String>,
    ) -> SubtrActorResult<()> {
        let key = factory.key();
        match visit_states.get(&key).copied() {
            Some(VisitState::Visiting) => {
                traversal_stack.push(key.clone());
                return SubtrActorError::new_result(
                    SubtrActorErrorVariant::StatsModuleDependencyCycle {
                        cycle: traversal_stack.clone(),
                    },
                );
            }
            Some(VisitState::Visited) => {
                if let Some(index) = resolved_indexes.get(&key) {
                    resolved[*index].emit |= emit;
                }
                return Ok(());
            }
            None => {}
        }

        visit_states.insert(key.clone(), VisitState::Visiting);
        traversal_stack.push(key.clone());

        for dependency in factory.dependencies() {
            visit(
                dependency,
                false,
                visit_states,
                resolved_indexes,
                resolved,
                traversal_stack,
            )?;
        }

        traversal_stack.pop();
        visit_states.insert(key.clone(), VisitState::Visited);
        resolved_indexes.insert(key.clone(), resolved.len());
        resolved.push(ResolvedStatsModuleFactory {
            key,
            name: factory.name(),
            emit,
            factory,
        });
        Ok(())
    }

    let mut visit_states = HashMap::new();
    let mut resolved_indexes = HashMap::new();
    let mut resolved = Vec::new();
    let mut traversal_stack = Vec::new();

    for module in modules {
        visit(
            module,
            true,
            &mut visit_states,
            &mut resolved_indexes,
            &mut resolved,
            &mut traversal_stack,
        )?;
    }

    let mut emitted_names: HashMap<&'static str, Vec<String>> = HashMap::new();
    for module in resolved.iter().filter(|module| module.emit) {
        emitted_names
            .entry(module.name)
            .or_default()
            .push(module.key.clone());
    }

    if let Some((name, keys)) = emitted_names.into_iter().find(|(_, keys)| keys.len() > 1) {
        return SubtrActorError::new_result(SubtrActorErrorVariant::DuplicateStatsModuleName {
            name: name.to_owned(),
            keys,
        });
    }

    Ok(resolved)
}
