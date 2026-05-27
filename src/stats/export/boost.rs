use crate::*;

use super::*;

#[path = "boost_collection.rs"]
mod boost_collection;
#[path = "boost_collection_counts.rs"]
mod boost_collection_counts;
#[path = "boost_labels.rs"]
mod boost_labels;
#[path = "boost_time.rs"]
mod boost_time;
#[path = "boost_usage.rs"]
mod boost_usage;

impl StatFieldProvider for BoostStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        self.visit_collection_stat_fields(visitor);
        self.visit_collection_count_stat_fields(visitor);
        self.visit_usage_stat_fields(visitor);
        self.visit_time_stat_fields(visitor);
        self.visit_labeled_stat_fields(visitor);
    }
}
