use std::collections::HashMap;

use chrono::{DateTime, Utc};

use super::{metadata::ImportanceLevel, recollection::RecollectionSource};

#[derive(Clone, Debug)]
pub struct SearchQuery {
    pub query: String,
    pub workspace_id: String,
    pub context: Option<HashMap<String, String>>,
    pub filters: Vec<QueryFilter>,
    pub sort_by: SortCriteria,
    pub top_k: usize, // Embeddingでの検索数
    pub limit: usize,
}

#[derive(Clone, Debug)]
pub enum QueryFilter {
    IncludeTags(Vec<String>),
    ExcludeTags(Vec<String>),
    Source(RecollectionSource),
    ImportanceMinimum(ImportanceLevel),
    CreatedAfter(DateTime<Utc>),
    CreatedBefore(DateTime<Utc>),
    ConfidenceMinimum(f32),
    Custom(HashMap<String, String>),
}

#[derive(Clone, Debug)]
pub enum SortCriteria {
    Relevance,
    CreationTime(SortOrder),
    Importance(SortOrder),
    Confidence(SortOrder),
}

#[derive(Clone, Debug)]
pub enum SortOrder {
    Ascending,
    Descending,
}
