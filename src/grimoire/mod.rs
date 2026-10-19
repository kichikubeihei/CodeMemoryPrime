pub mod contract;
pub mod sentinels;
pub mod sandbox;
pub mod collective;
pub mod gauntlet;

pub use contract::{AlgorithmContract, ComplexityLimits};
pub use sentinels::{AntiCheatSentinel, SentinelVerdict};
pub use sandbox::{ExecutionTelemetry, VerificationBackend, ProcessIsolatedBackend, LocalContainerBackend, MicroVmBackend};
pub use collective::{VerifiedSpell, StrikeVerdict, inscribe_winning_spell, query_collective_spells, record_strike, check_ban_status};
pub use gauntlet::{GrimoireGauntlet, GauntletReport, GauntletStatus};
