//! Which network a chain is configured against, and whether a deployment's
//! chains agree with each other.
//!
//! This exists because of a real defect. The browser read Solana balances on
//! devnet (from its own `VITE_SOLANA_RPC_URL`) while the backend verified
//! against mainnet-beta, and the Ethereum chain id served to the wallet was
//! mainnet. Each value was individually plausible; nothing compared them. A
//! payment checked on one network and verified on another can never confirm,
//! and in the opposite direction -- a wallet pointed at mainnet while the
//! operator believes they are testing -- it spends real money.
//!
//! Chain ids and RPC endpoints are the only ground truth available, so the realm
//! is derived from them rather than configured separately. A second setting
//! someone can forget to change is the bug, not the fix.

use super::Chain;

/// Mainnet (real value) or a test network (valueless).
///
/// Deliberately two-valued. The distinction that matters operationally is
/// whether a mistake costs real money, not which particular testnet is in use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkRealm {
    Mainnet,
    Test,
}

impl NetworkRealm {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mainnet => "mainnet",
            Self::Test => "test",
        }
    }
}

/// A chain's concrete network, with the human-facing name the API reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Network {
    pub realm: NetworkRealm,
    /// Canonical name, e.g. "mainnet-beta", "devnet", "sepolia", "bsc-testnet".
    pub name: String,
}

impl Network {
    fn new(realm: NetworkRealm, name: &str) -> Self {
        Self {
            realm,
            name: name.to_string(),
        }
    }
}

/// EIP-155 chain id to network. Unknown ids are treated as test networks on
/// purpose: an unrecognised id is far more likely to be a local or new testnet
/// than a mainnet, and guessing "mainnet" here is the guess that loses money.
pub fn evm_network(chain_id: u64) -> Network {
    match chain_id {
        1 => Network::new(NetworkRealm::Mainnet, "ethereum-mainnet"),
        56 => Network::new(NetworkRealm::Mainnet, "bsc-mainnet"),
        11155111 => Network::new(NetworkRealm::Test, "sepolia"),
        17000 => Network::new(NetworkRealm::Test, "holesky"),
        97 => Network::new(NetworkRealm::Test, "bsc-testnet"),
        31337 | 1337 => Network::new(NetworkRealm::Test, "localhost"),
        other => Network::new(NetworkRealm::Test, &format!("evm-{}", other)),
    }
}

/// Solana cluster from its RPC endpoint.
///
/// Matched on substrings because the endpoint is frequently a provider URL
/// (QuickNode, Helius, Alchemy) rather than `api.*.solana.com`, and those encode
/// the cluster in the host or path. An endpoint that names no cluster is
/// reported as unknown rather than assumed -- see `realm_of` for why that is not
/// silently treated as a test network.
pub fn solana_network(rpc_url: &str) -> Option<Network> {
    let url = rpc_url.to_ascii_lowercase();
    if url.contains("devnet") {
        Some(Network::new(NetworkRealm::Test, "devnet"))
    } else if url.contains("testnet") {
        Some(Network::new(NetworkRealm::Test, "testnet"))
    } else if url.contains("localhost") || url.contains("127.0.0.1") {
        Some(Network::new(NetworkRealm::Test, "localnet"))
    } else if url.contains("mainnet") {
        Some(Network::new(NetworkRealm::Mainnet, "mainnet-beta"))
    } else {
        None
    }
}

/// Bitcoin network from the explorer base URL. Blockstream and mempool.space
/// both expose testnet under a `/testnet` path segment.
pub fn bitcoin_network(api_base: &str) -> Option<Network> {
    let url = api_base.to_ascii_lowercase();
    if url.contains("/testnet") || url.contains("/signet") {
        Some(Network::new(NetworkRealm::Test, "bitcoin-testnet"))
    } else if url.contains("blockstream.info") || url.contains("mempool.space") {
        Some(Network::new(NetworkRealm::Mainnet, "bitcoin-mainnet"))
    } else {
        None
    }
}

/// One configured chain, as the consistency check sees it.
pub struct ConfiguredChain {
    pub chain: Chain,
    pub network: Option<Network>,
}

/// Reject a deployment whose chains do not agree about mainnet versus test.
///
/// Only chains the operator actually enabled are passed in, so a half-configured
/// deployment is not failed for networks it never advertises. A chain whose
/// network could not be determined is reported separately rather than assumed:
/// an unrecognised endpoint beside a mainnet one is exactly the case where
/// guessing is most expensive.
pub fn check_realm_consistency(chains: &[ConfiguredChain]) -> Result<(), String> {
    let mut mainnet: Vec<&str> = Vec::new();
    let mut test: Vec<&str> = Vec::new();
    let mut unknown: Vec<&str> = Vec::new();

    for configured in chains {
        match &configured.network {
            Some(n) if n.realm == NetworkRealm::Mainnet => mainnet.push(configured.chain.as_str()),
            Some(_) => test.push(configured.chain.as_str()),
            None => unknown.push(configured.chain.as_str()),
        }
    }

    if !mainnet.is_empty() && !test.is_empty() {
        return Err(format!(
            "Chains disagree about which network they are on: {} on mainnet, {} on a test network. \
             A payment made on one and verified on the other can never confirm, and a wallet \
             pointed at mainnet during testing spends real funds. Put every configured chain on \
             the same network.",
            mainnet.join(", "),
            test.join(", ")
        ));
    }

    if !mainnet.is_empty() && !unknown.is_empty() {
        return Err(format!(
            "{} is on mainnet but the network for {} could not be determined from its endpoint. \
             Refusing to guess next to real funds -- use an endpoint whose URL names its network.",
            mainnet.join(", "),
            unknown.join(", ")
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evm_mainnet_ids_are_mainnet() {
        assert_eq!(evm_network(1).realm, NetworkRealm::Mainnet);
        assert_eq!(evm_network(56).realm, NetworkRealm::Mainnet);
    }

    #[test]
    fn evm_testnet_ids_are_test() {
        assert_eq!(evm_network(11155111).realm, NetworkRealm::Test);
        assert_eq!(evm_network(97).realm, NetworkRealm::Test);
        assert_eq!(evm_network(31337).realm, NetworkRealm::Test);
    }

    #[test]
    fn an_unknown_evm_id_is_treated_as_a_test_network() {
        // Guessing "mainnet" for an unrecognised id is the guess that loses money.
        assert_eq!(evm_network(424242).realm, NetworkRealm::Test);
    }

    #[test]
    fn solana_clusters_are_read_from_the_endpoint() {
        assert_eq!(
            solana_network("https://api.devnet.solana.com")
                .unwrap()
                .realm,
            NetworkRealm::Test
        );
        assert_eq!(
            solana_network("https://api.mainnet-beta.solana.com")
                .unwrap()
                .realm,
            NetworkRealm::Mainnet
        );
        assert_eq!(
            solana_network("http://127.0.0.1:8899").unwrap().name,
            "localnet"
        );
    }

    #[test]
    fn a_provider_endpoint_naming_no_cluster_is_unknown() {
        // A QuickNode-style URL whose path is the auth token says nothing about
        // the cluster, and must not be assumed either way.
        assert!(solana_network("https://example.quiknode.pro/abc123/").is_none());
    }

    #[test]
    fn bitcoin_testnet_is_recognised_by_path() {
        assert_eq!(
            bitcoin_network("https://blockstream.info/testnet/api")
                .unwrap()
                .realm,
            NetworkRealm::Test
        );
        assert_eq!(
            bitcoin_network("https://blockstream.info/api")
                .unwrap()
                .realm,
            NetworkRealm::Mainnet
        );
    }

    #[test]
    fn mixing_mainnet_and_test_is_rejected() {
        // The exact defect this type exists for: Solana on devnet, Ethereum on
        // mainnet, each individually plausible.
        let err = check_realm_consistency(&[
            ConfiguredChain {
                chain: Chain::Solana,
                network: solana_network("https://api.devnet.solana.com"),
            },
            ConfiguredChain {
                chain: Chain::Ethereum,
                network: Some(evm_network(1)),
            },
        ])
        .unwrap_err();
        assert!(err.contains("Solana"), "{}", err);
        assert!(err.contains("Ethereum"), "{}", err);
    }

    #[test]
    fn an_all_test_deployment_is_accepted() {
        check_realm_consistency(&[
            ConfiguredChain {
                chain: Chain::Solana,
                network: solana_network("https://api.devnet.solana.com"),
            },
            ConfiguredChain {
                chain: Chain::Ethereum,
                network: Some(evm_network(11155111)),
            },
        ])
        .unwrap();
    }

    #[test]
    fn an_all_mainnet_deployment_is_accepted() {
        check_realm_consistency(&[
            ConfiguredChain {
                chain: Chain::Solana,
                network: solana_network("https://api.mainnet-beta.solana.com"),
            },
            ConfiguredChain {
                chain: Chain::Bitcoin,
                network: bitcoin_network("https://blockstream.info/api"),
            },
        ])
        .unwrap();
    }

    #[test]
    fn an_unknown_endpoint_beside_mainnet_is_rejected() {
        let err = check_realm_consistency(&[
            ConfiguredChain {
                chain: Chain::Ethereum,
                network: Some(evm_network(1)),
            },
            ConfiguredChain {
                chain: Chain::Solana,
                network: solana_network("https://example.quiknode.pro/abc123/"),
            },
        ])
        .unwrap_err();
        assert!(err.contains("could not be determined"), "{}", err);
    }

    #[test]
    fn an_unknown_endpoint_alone_is_allowed() {
        // Nothing to contradict, and failing here would block every private
        // RPC provider on a test deployment.
        check_realm_consistency(&[ConfiguredChain {
            chain: Chain::Solana,
            network: solana_network("https://example.quiknode.pro/abc123/"),
        }])
        .unwrap();
    }
}
