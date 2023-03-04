use std::collections::HashMap;

use iso8601_timestamp::Timestamp;
use serde::{Deserialize, Serialize};

/// Index access information
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct IndexAccess {
    /// Operations since timestamp
    ops: i32,

    /// Timestamp at which data keeping begun
    since: Timestamp,
}

/// Collection index
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct Index {
    /// Index name
    name: String,

    /// Access information
    accesses: IndexAccess,
}

/// Histogram entry
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct LatencyHistogramEntry {
    /// Time
    micros: i64,

    /// Count
    count: i64,
}

/// Collection latency stats
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct LatencyStats {
    /// Total operations
    ops: i64,

    /// Timestamp at which data keeping begun
    latency: i64,

    /// Histogram representation of latency data
    histogram: Vec<LatencyHistogramEntry>,
}

/// Collection storage stats
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StorageStats {
    /// Uncompressed data size
    size: i64,

    /// Data size on disk
    storage_size: i64,

    /// Total size of all indexes
    total_index_size: i64,

    /// Sum of storage size and total index size
    total_size: i64,

    /// Individual index sizes
    index_sizes: HashMap<String, i64>,

    /// Number of documents in collection
    count: i64,

    /// Average size of each document
    avg_obj_size: i64,
}

/// Query collection scan stats
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CollectionScans {
    /// Number of total collection scans
    total: i64,

    /// Number of total collection scans not using a tailable cursor
    non_tailable: i64,
}

/// Collection query execution stats
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QueryExecStats {
    /// Stats regarding collection scans
    collection_scans: CollectionScans,
}

/// Collection stats
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CollectionStats {
    /// Namespace
    ns: String,

    /// Local time
    local_time: Timestamp,

    /// Latency stats
    latency_stats: HashMap<String, LatencyStats>,

    /// Query exec stats
    query_exec_stats: QueryExecStats,

    /// Number of documents in collection
    count: u64,
}

/// Server Stats
#[derive(Serialize, JsonSchema, Debug)]
pub struct Stats {
    /// Index usage information
    pub indices: HashMap<String, Vec<Index>>,

    /// Collection stats
    pub coll_stats: HashMap<String, CollectionStats>,
}
