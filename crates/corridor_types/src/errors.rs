use soroban_sdk::contracterror;

/// Every error the Corridor contracts can return. Numbers are stable — the
/// relayer and SDK match on them (e.g. `#4` = "post a first root",
/// `#15` = "never retry this epoch").
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotAuthorized = 3,
    PolicyNotFound = 4,
    PolicyPaused = 5,
    PolicyExists = 6,
    RootMismatch = 7,
    CorridorMismatch = 8,
    MinTierMismatch = 9,
    StaleProofTime = 10,
    ProofInvalid = 11,
    NullifierUsed = 12,
    IssuerNotAccepted = 13,
    BadPublicInputs = 14,
    RootEpochRegression = 15,
    RelayerNotAllowed = 16,
    NoPendingAdmin = 17,
    DisclosureMissing = 18,
}
