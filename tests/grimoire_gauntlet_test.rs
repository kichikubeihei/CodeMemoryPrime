use rusqlite::Connection;
use codememory_prime::grimoire::{
    AlgorithmContract, AntiCheatSentinel, GrimoireGauntlet, GauntletStatus,
    SentinelVerdict, query_collective_spells, record_strike, check_ban_status
};

#[test]
fn test_contract_catalog() {
    let catalog = AlgorithmContract::catalog();
    assert!(catalog.len() >= 10, "Expected at least 10 foundational contracts in Grimoire");

    let kd_tree = AlgorithmContract::find_by_id("spatial_kd_tree").expect("spatial_kd_tree missing");
    assert_eq!(kd_tree.category, "spatial_indexing");
    assert!(kd_tree.supported_languages.contains(&"python".to_string()));
    assert!(kd_tree.supported_languages.contains(&"typescript".to_string()));
    assert!(kd_tree.preconditions.len() >= 3);
    assert!(kd_tree.postconditions.len() >= 3);
}

#[test]
fn test_sentinel_ast_firewall_rejection() {
    // 1. Python OS import exploit
    let exploit_py = "import os\ndef solve(): os.system('echo hack')";
    let verdict = AntiCheatSentinel::inspect_static_ast(exploit_py, "python");
    assert!(matches!(verdict, SentinelVerdict::StrikeableViolation { .. }), "Expected strike for banned OS import");

    // 2. JavaScript child_process exploit
    let exploit_js = "const cp = require('child_process'); function solve() { cp.execSync('whoami'); }";
    let verdict_js = AntiCheatSentinel::inspect_static_ast(exploit_js, "typescript");
    assert!(matches!(verdict_js, SentinelVerdict::StrikeableViolation { .. }), "Expected strike for child_process");

    // 3. Monkey-patch cheat
    let cheat_py = "import unittest\nunittest.TestCase.assertTrue = lambda *args: True\nclass Solution: pass";
    let verdict_cheat = AntiCheatSentinel::inspect_static_ast(cheat_py, "python");
    assert!(matches!(verdict_cheat, SentinelVerdict::StrikeableViolation { .. }), "Expected strike for monkey-patch");
}

#[test]
fn test_sentinel_asymptotic_curve_fit() {
    // O(1) expected: small time growth is fine
    let good_o1 = vec![(10, 1.2), (1000, 1.4)];
    assert!(AntiCheatSentinel::verify_asymptotic_curve(&good_o1, "O(1)").is_ok());

    // O(1) expected, but time exploded 50x: rejected
    let bad_o1 = vec![(10, 1.0), (1000, 55.0)];
    assert!(AntiCheatSentinel::verify_asymptotic_curve(&bad_o1, "O(1)").is_err());

    // O(N log N) expected, but scaled by 1,000,000x (quadratic): rejected
    let bad_nlogn = vec![(10, 1.0), (10000, 100000.0)];
    assert!(AntiCheatSentinel::verify_asymptotic_curve(&bad_nlogn, "O(N log N)").is_err());
}

#[test]
fn test_three_strikes_ban_enforcement() {
    let conn = Connection::open_in_memory().unwrap();
    let ip = "192.168.1.100";
    let key = "client-ed25519-bad-actor";

    assert_eq!(check_ban_status(&conn, ip, key).unwrap(), false);

    // Strike 1
    let v1 = record_strike(&conn, ip, key, "Banned import os.system").unwrap();
    assert_eq!(v1.strike_count, 1);
    assert_eq!(v1.is_banned, false);

    // Strike 2
    let v2 = record_strike(&conn, ip, key, "Monkey-patch unittest").unwrap();
    assert_eq!(v2.strike_count, 2);
    assert_eq!(v2.is_banned, false);

    // Strike 3
    let v3 = record_strike(&conn, ip, key, "Subprocess reverse shell").unwrap();
    assert_eq!(v3.strike_count, 3);
    assert_eq!(v3.is_banned, true);

    // Check ban status is now true
    assert_eq!(check_ban_status(&conn, ip, key).unwrap(), true);

    // Gauntlet run should immediately reject banned client
    let report = GrimoireGauntlet::run_challenge(
        &conn,
        "lru_cache",
        "python",
        "class LRUCache: pass",
        "bad-actor",
        ip,
        key,
        None,
        false
    );
    assert!(matches!(report.status, GauntletStatus::ClientBanned));
}

#[test]
fn test_gauntlet_honest_lru_cache_verification_and_harvest() {
    let conn = Connection::open_in_memory().unwrap();
    let ip = "127.0.0.1";
    let key = "client-ed25519-honest-researcher";

    // Clean, legitimate Python LRU Cache implementation
    let valid_lru_python = r#"
class Node:
    def __init__(self, key=0, val=0):
        self.key = key
        self.val = val
        self.prev = None
        self.next = None

class LRUCache:
    def __init__(self, capacity: int):
        self.cap = capacity
        self.cache = {}
        self.head = Node()
        self.tail = Node()
        self.head.next = self.tail
        self.tail.prev = self.head

    def _remove(self, node):
        prev = node.prev
        nxt = node.next
        prev.next = nxt
        nxt.prev = prev

    def _add(self, node):
        node.prev = self.head
        node.next = self.head.next
        self.head.next.prev = node
        self.head.next = node

    def get(self, key: int):
        if key in self.cache:
            node = self.cache[key]
            self._remove(node)
            self._add(node)
            return node.val
        return None

    def put(self, key: int, val: str):
        if key in self.cache:
            self._remove(self.cache[key])
        node = Node(key, val)
        self._add(node)
        self.cache[key] = node
        if len(self.cache) > self.cap:
            lru = self.tail.prev
            self._remove(lru)
            del self.cache[lru.key]
"#;

    let report = GrimoireGauntlet::run_challenge(
        &conn,
        "lru_cache",
        "python",
        valid_lru_python,
        "claude-3.7-sonnet",
        ip,
        key,
        None,
        true // auto_harvest!
    );

    assert!(
        matches!(report.status, GauntletStatus::PassedNewRecord | GauntletStatus::PassedVerified),
        "Expected successful verification of clean LRU cache. Report: {:?}", report
    );
    assert_eq!(report.contracts_passed, 5);
    assert!(report.inscribed_spell.is_some(), "Expected algorithm to be inscribed in collective");

    // Verify it can be queried from the collective
    let spells = query_collective_spells(&conn, Some("lru_cache"), Some("python")).unwrap();
    assert_eq!(spells.len(), 1);
    assert_eq!(spells[0].author_alias, "claude-3.7-sonnet");
    assert!(!spells[0].hmac_signature.is_empty());
}
