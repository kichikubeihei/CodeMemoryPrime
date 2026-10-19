use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComplexityLimits {
    pub time_complexity: String,
    pub space_complexity: String,
    pub benchmark_baseline_us: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlgorithmContract {
    pub algorithm_id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub supported_languages: Vec<String>,
    pub preconditions: Vec<String>,
    pub postconditions: Vec<String>,
    pub complexity_limits: ComplexityLimits,
}

impl AlgorithmContract {
    pub fn catalog() -> Vec<AlgorithmContract> {
        vec![
            AlgorithmContract {
                algorithm_id: "spatial_kd_tree".to_string(),
                name: "2D/3D K-Dimensional Tree".to_string(),
                category: "spatial_indexing".to_string(),
                description: "Space-partitioning data structure for sub-millisecond nearest-neighbor searches in Euclidean metric space.".to_string(),
                supported_languages: vec!["python".to_string(), "typescript".to_string(), "go".to_string(), "dart".to_string(), "rust".to_string()],
                preconditions: vec![
                    "Points array must contain N >= 1 points".to_string(),
                    "Each point coordinate dimension must match tree dimensionality K".to_string(),
                    "Coordinates must be valid finite floats (no NaN or infinity)".to_string(),
                ],
                postconditions: vec![
                    "nearest_neighbor(query) must return point P in tree minimizing Euclidean distance".to_string(),
                    "range_search(box) must return exactly all points contained within bounds".to_string(),
                    "Tree must maintain invariant: left subtree <= median coordinate, right subtree > median".to_string(),
                ],
                complexity_limits: ComplexityLimits {
                    time_complexity: "O(log N) query, O(N log N) build".to_string(),
                    space_complexity: "O(N)".to_string(),
                    benchmark_baseline_us: 12.5,
                },
            },
            AlgorithmContract {
                algorithm_id: "consistent_hashing".to_string(),
                name: "Consistent Hashing Ring with Virtual Nodes".to_string(),
                category: "distributed_systems".to_string(),
                description: "Distributed ring topology minimizing key reassignment during cluster node addition or removal.".to_string(),
                supported_languages: vec!["python".to_string(), "typescript".to_string(), "go".to_string(), "dart".to_string(), "rust".to_string()],
                preconditions: vec![
                    "Virtual node replication factor V >= 10".to_string(),
                    "Node identifiers and keys must be non-empty strings".to_string(),
                ],
                postconditions: vec![
                    "get_node(key) must deterministically route to first node clockwise on ring".to_string(),
                    "Adding node K must reassign at most K/N keys (monotonicity invariant)".to_string(),
                    "Removing node must distribute keys evenly to surviving adjacent nodes".to_string(),
                ],
                complexity_limits: ComplexityLimits {
                    time_complexity: "O(log(N * V)) query".to_string(),
                    space_complexity: "O(N * V)".to_string(),
                    benchmark_baseline_us: 4.8,
                },
            },
            AlgorithmContract {
                algorithm_id: "bloom_filter".to_string(),
                name: "Optimal K-Hash Bloom Filter".to_string(),
                category: "probabilistic".to_string(),
                description: "Space-efficient probabilistic set membership testing with zero false negatives.".to_string(),
                supported_languages: vec!["python".to_string(), "typescript".to_string(), "go".to_string(), "dart".to_string(), "rust".to_string()],
                preconditions: vec![
                    "Target false positive rate 0.0 < P < 1.0".to_string(),
                    "Expected element capacity N >= 1".to_string(),
                ],
                postconditions: vec![
                    "False negative probability must be exactly 0.0 (if added, contains(x) is true)".to_string(),
                    "Empirical false positive rate on uninserted elements must not exceed P * 1.2".to_string(),
                    "Optimal bit array size M and hash count K calculated via standard formulas".to_string(),
                ],
                complexity_limits: ComplexityLimits {
                    time_complexity: "O(K) insert and lookup".to_string(),
                    space_complexity: "O(M) bits".to_string(),
                    benchmark_baseline_us: 1.8,
                },
            },
            AlgorithmContract {
                algorithm_id: "lru_cache".to_string(),
                name: "O(1) Concurrent-Safe LRU Cache".to_string(),
                category: "concurrency".to_string(),
                description: "Least Recently Used cache combining doubly-linked list with hash map for O(1) eviction.".to_string(),
                supported_languages: vec!["python".to_string(), "typescript".to_string(), "go".to_string(), "dart".to_string(), "rust".to_string()],
                preconditions: vec![
                    "Capacity C >= 1".to_string(),
                    "Keys must be hashable and comparable".to_string(),
                ],
                postconditions: vec![
                    "get(k) moves key to head of recency list in O(1)".to_string(),
                    "put(k, v) on full cache evicts exact tail element in O(1)".to_string(),
                    "Cache size must never exceed C".to_string(),
                ],
                complexity_limits: ComplexityLimits {
                    time_complexity: "O(1) get, put, evict".to_string(),
                    space_complexity: "O(C)".to_string(),
                    benchmark_baseline_us: 2.1,
                },
            },
            AlgorithmContract {
                algorithm_id: "rate_limiter_token_bucket".to_string(),
                name: "High-Throughput Token Bucket Rate Limiter".to_string(),
                category: "concurrency".to_string(),
                description: "Traffic-shaping token bucket allowing burst capacity with smooth replenishment rate.".to_string(),
                supported_languages: vec!["python".to_string(), "typescript".to_string(), "go".to_string(), "dart".to_string(), "rust".to_string()],
                preconditions: vec![
                    "Capacity burst_limit >= 1".to_string(),
                    "Refill rate tokens_per_sec > 0.0".to_string(),
                ],
                postconditions: vec![
                    "allow(tokens) returns true if sufficient tokens available, decrementing bucket".to_string(),
                    "allow(tokens) returns false without side effects if insufficient".to_string(),
                    "Tokens replenish linearly over elapsed duration without floating point drift".to_string(),
                ],
                complexity_limits: ComplexityLimits {
                    time_complexity: "O(1) check and consume".to_string(),
                    space_complexity: "O(1)".to_string(),
                    benchmark_baseline_us: 0.8,
                },
            },
            AlgorithmContract {
                algorithm_id: "aho_corasick".to_string(),
                name: "Aho-Corasick Multi-Pattern Automaton".to_string(),
                category: "string_search".to_string(),
                description: "Trie with failure links for linear-time simultaneous dictionary matching.".to_string(),
                supported_languages: vec!["python".to_string(), "typescript".to_string(), "go".to_string(), "dart".to_string(), "rust".to_string()],
                preconditions: vec![
                    "Pattern dictionary size P >= 1".to_string(),
                    "Input text length T >= 0".to_string(),
                ],
                postconditions: vec![
                    "find_all(text) returns exact start/end offsets of all pattern occurrences".to_string(),
                    "Overlapping patterns must all be reported".to_string(),
                    "Execution must match in single text pass via deterministic failure transitions".to_string(),
                ],
                complexity_limits: ComplexityLimits {
                    time_complexity: "O(T + M) where M is matches count".to_string(),
                    space_complexity: "O(Total Pattern Characters)".to_string(),
                    benchmark_baseline_us: 18.0,
                },
            },
            AlgorithmContract {
                algorithm_id: "tarjan_scc".to_string(),
                name: "Tarjan's Strongly Connected Components".to_string(),
                category: "graph_theory".to_string(),
                description: "Single-pass DFS algorithm partitioning directed graphs into maximal strongly connected subgraphs.".to_string(),
                supported_languages: vec!["python".to_string(), "typescript".to_string(), "go".to_string(), "dart".to_string(), "rust".to_string()],
                preconditions: vec![
                    "Graph represented as adjacency list with V vertices and E directed edges".to_string(),
                ],
                postconditions: vec![
                    "Partitions all vertices into mutually disjoint SCC subsets".to_string(),
                    "For every pair u, v in same SCC, a directed path exists u -> v and v -> u".to_string(),
                    "Condensation graph of SCCs forms a valid Directed Acyclic Graph (DAG)".to_string(),
                ],
                complexity_limits: ComplexityLimits {
                    time_complexity: "O(V + E)".to_string(),
                    space_complexity: "O(V)".to_string(),
                    benchmark_baseline_us: 15.2,
                },
            },
            AlgorithmContract {
                algorithm_id: "suffix_automaton".to_string(),
                name: "Minimal Suffix Automaton (DAWG)".to_string(),
                category: "string_search".to_string(),
                description: "Directed acyclic word graph recognizing all suffixes of a string with minimal states.".to_string(),
                supported_languages: vec!["python".to_string(), "typescript".to_string(), "go".to_string(), "dart".to_string(), "rust".to_string()],
                preconditions: vec![
                    "String of length N >= 1".to_string(),
                ],
                postconditions: vec![
                    "Number of states must not exceed 2N - 1".to_string(),
                    "Number of transitions must not exceed 3N - 4".to_string(),
                    "accepts(substr) returns true iff substr is a substring of original string".to_string(),
                ],
                complexity_limits: ComplexityLimits {
                    time_complexity: "O(N) construction, O(K) substring check".to_string(),
                    space_complexity: "O(N)".to_string(),
                    benchmark_baseline_us: 22.0,
                },
            },
            AlgorithmContract {
                algorithm_id: "hyperloglog".to_string(),
                name: "HyperLogLog Cardinality Estimator".to_string(),
                category: "probabilistic".to_string(),
                description: "Near-optimal space distinct count estimator using stochastic averaging of harmonic means.".to_string(),
                supported_languages: vec!["python".to_string(), "typescript".to_string(), "go".to_string(), "dart".to_string(), "rust".to_string()],
                preconditions: vec![
                    "Precision register parameter 4 <= B <= 16".to_string(),
                ],
                postconditions: vec![
                    "Standard error of estimate bounded by 1.04 / sqrt(2^B)".to_string(),
                    "merge(other_hll) yields exact bitwise union with identical precision".to_string(),
                ],
                complexity_limits: ComplexityLimits {
                    time_complexity: "O(1) add and estimate".to_string(),
                    space_complexity: "O(2^B) register bytes".to_string(),
                    benchmark_baseline_us: 3.2,
                },
            },
            AlgorithmContract {
                algorithm_id: "dijkstra_indexed_heap".to_string(),
                name: "Dijkstra with Indexed Binary Heap".to_string(),
                category: "graph_theory".to_string(),
                description: "Shortest path search on non-negative weighted graphs with O(log V) decrease-key operations.".to_string(),
                supported_languages: vec!["python".to_string(), "typescript".to_string(), "go".to_string(), "dart".to_string(), "rust".to_string()],
                preconditions: vec![
                    "Graph edges must have non-negative weights W >= 0".to_string(),
                    "Source node must exist in graph".to_string(),
                ],
                postconditions: vec![
                    "dist[v] equals minimum cumulative weight path from source to v".to_string(),
                    "Reconstructed path contains no cycles".to_string(),
                    "Unreachable nodes marked with infinity distance".to_string(),
                ],
                complexity_limits: ComplexityLimits {
                    time_complexity: "O((V + E) log V)".to_string(),
                    space_complexity: "O(V)".to_string(),
                    benchmark_baseline_us: 25.0,
                },
            },
        ]
    }

    pub fn find_by_id(id: &str) -> Option<AlgorithmContract> {
        Self::catalog().into_iter().find(|c| c.algorithm_id == id)
    }

    pub fn find_by_category(cat: &str) -> Vec<AlgorithmContract> {
        Self::catalog().into_iter().filter(|c| c.category == cat).collect()
    }
}
