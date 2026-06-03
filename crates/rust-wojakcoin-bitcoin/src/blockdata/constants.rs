// Written in 2014 by Andrew Poelstra <apoelstra@wpsoftware.net>
// SPDX-License-Identifier: CC0-1.0

//! Blockdata constants.
//!
//! This module provides various constants relating to the blockchain and
//! consensus code. In particular, it defines the genesis block and its
//! single transaction.
//!

use crate::prelude::*;

use core::default::Default;

use crate::hashes::hex::{self, HexIterator, FromHex};
use crate::hashes::{Hash, sha256d};
use crate::blockdata::opcodes;
use crate::blockdata::script;
use crate::blockdata::locktime::PackedLockTime;
use crate::blockdata::transaction::{OutPoint, Transaction, TxOut, TxIn, Sequence};
use crate::blockdata::block::{Block, BlockHeader};
use crate::blockdata::witness::Witness;
use crate::network::constants::Network;
use crate::util::uint::Uint256;
use crate::internal_macros::{impl_array_newtype, impl_bytes_newtype};

/// How many satoshis are in "one bitcoin"
pub const COIN_VALUE: u64 = 100_000_000;
/// How many seconds between blocks we expect on average
pub const TARGET_BLOCK_SPACING: u32 = 600;
/// How many blocks between diffchanges
pub const DIFFCHANGE_INTERVAL: u32 = 2016;
/// How much time on average should occur between diffchanges
pub const DIFFCHANGE_TIMESPAN: u32 = 14 * 24 * 3600;
/// The maximum allowed weight for a block, see BIP 141 (network rule)
pub const MAX_BLOCK_WEIGHT: u32 = 4_000_000;
/// The minimum transaction weight for a valid serialized transaction
pub const MIN_TRANSACTION_WEIGHT: u32 = 4 * 60;
/// The factor that non-witness serialization data is multiplied by during weight calculation
pub const WITNESS_SCALE_FACTOR: usize = 4;
/// The maximum allowed number of signature check operations in a block
pub const MAX_BLOCK_SIGOPS_COST: i64 = 80_000;
/// Mainnet (bitcoin) pubkey address prefix.
pub const PUBKEY_ADDRESS_PREFIX_MAIN: u8 = 0x49;
/// Mainnet (bitcoin) script address prefix.
pub const SCRIPT_ADDRESS_PREFIX_MAIN: u8 = 0x05;
/// Test (tesnet, signet, regtest) pubkey address prefix.
pub const PUBKEY_ADDRESS_PREFIX_TEST: u8 = 0x71; 
/// Test (tesnet, signet, regtest) script address prefix.
pub const SCRIPT_ADDRESS_PREFIX_TEST: u8 = 0xc4;
/// The maximum allowed script size.
pub const MAX_SCRIPT_ELEMENT_SIZE: usize = 520;
/// How may blocks between halvings.
pub const SUBSIDY_HALVING_INTERVAL: u32 = 210_000;
/// Maximum allowed value for an integer in Script.
pub const MAX_SCRIPTNUM_VALUE: u32 = 0x80000000; // 2^31

/// In Bitcoind this is insanely described as ~((u256)0 >> 32)
pub fn max_target(_: Network) -> Uint256 {
    Uint256::from_u64(0xFFFF).unwrap() << 208
}

/// The maximum value allowed in an output (useful for sanity checking,
/// since keeping everything below this value should prevent overflows
/// if you are doing anything remotely sane with monetary values).
pub fn max_money(_: Network) -> u64 {
    21_000_000 * COIN_VALUE
}

/// Constructs and returns the coinbase (and only) transaction of the Bitcoin genesis block
fn bitcoin_genesis_tx() -> Transaction {
    // Base
    let mut ret = Transaction {
        version: 1,
        lock_time: PackedLockTime::ZERO,
        input: vec![],
        output: vec![],
    };

    // Inputs
    let in_script = script::Builder::new().push_scriptint(486604799)
                                          .push_scriptint(4)
                                          .push_slice(b"The Times 03/Jan/2009 Chancellor on brink of second bailout for banks")
                                          .into_script();
    ret.input.push(TxIn {
        previous_output: OutPoint::null(),
        script_sig: in_script,
        sequence: Sequence::MAX,
        witness: Witness::default(),
    });

    // Outputs
    let script_bytes: Result<Vec<u8>, hex::Error> =
        HexIterator::new("04678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5f").unwrap()
            .collect();
    let out_script = script::Builder::new()
        .push_slice(script_bytes.unwrap().as_slice())
        .push_opcode(opcodes::all::OP_CHECKSIG)
        .into_script();
    ret.output.push(TxOut {
        value: 50 * COIN_VALUE,
        script_pubkey: out_script
    });

    // end
    ret
}

/// Constructs and returns the coinbase (and only) transaction of the Pepecoin genesis block
fn pepecoin_genesis_tx() -> Transaction {
    use crate::consensus::encode::deserialize;
    let tx_hex = "01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff5104ffff001d01044957534a20312f32322f3234202d204665642052657669657720436c656172732043656e7472616c2042616e6b204f6666696369616c73206f662056696f6c6174696e672052756c6573ffffffff010058850c0200000043410436d04f40a76a1094ea10b14a513b62bfd0b47472dda1c25aa9cf8266e53f3c4353680146177f8a3b328ed2c6e02f2b8e051d9d5ffc61a4e6ccabd03409109a5aac00000000";
    deserialize(&Vec::from_hex(tx_hex).unwrap()).unwrap()
}

/// Constructs and returns the genesis block
pub fn genesis_block(network: Network) -> Block {
    match network {
        Network::Bitcoin => {
            let txdata = vec![pepecoin_genesis_tx()];
            let merkle_root = sha256d::Hash::from_hex("d22a1ba59a39cbd5904624933efb822c8baa121f97060c4cc9ea2f00a4bc6512").unwrap().into();
            Block {
                header: BlockHeader {
                    version: 1,
                    prev_blockhash: Hash::all_zeros(),
                    merkle_root,
                    time: 1705975200,
                    bits: 0x1e0ffff0,
                    nonce: 427444
                },
                txdata,
            }
        }
        Network::Testnet => {
            let txdata = vec![bitcoin_genesis_tx()];
            let hash: sha256d::Hash = txdata[0].txid().into();
            let merkle_root = hash.into();
            Block {
                header: BlockHeader {
                    version: 1,
                    prev_blockhash: Hash::all_zeros(),
                    merkle_root,
                    time: 1296688602,
                    bits: 0x1d00ffff,
                    nonce: 414098458
                },
                txdata,
            }
        }
        Network::Signet => {
            let txdata = vec![bitcoin_genesis_tx()];
            let hash: sha256d::Hash = txdata[0].txid().into();
            let merkle_root = hash.into();
            Block {
                header: BlockHeader {
                    version: 1,
                    prev_blockhash: Hash::all_zeros(),
                    merkle_root,
                    time: 1598918400,
                    bits: 0x1e0377ae,
                    nonce: 52613770
                },
                txdata,
            }
        }
        Network::Regtest => {
            let txdata = vec![bitcoin_genesis_tx()];
            let hash: sha256d::Hash = txdata[0].txid().into();
            let merkle_root = hash.into();
            Block {
                header: BlockHeader {
                    version: 1,
                    prev_blockhash: Hash::all_zeros(),
                    merkle_root,
                    time: 1296688602,
                    bits: 0x207fffff,
                    nonce: 2
                },
                txdata,
            }
        }
    }
}

// Mainnet value can be verified at https://github.com/lightning/bolts/blob/master/00-introduction.md
const GENESIS_BLOCK_HASH_BITCOIN: [u8; 32] = [170, 95, 42, 140, 127, 245, 145, 176, 92, 151, 255, 147, 219, 170, 141, 131, 160, 233, 236, 66, 138, 108, 55, 101, 137, 212, 184, 72, 12, 28, 152, 55];
const GENESIS_BLOCK_HASH_TESTNET: [u8; 32] = [67, 73, 127, 215, 248, 38, 149, 113, 8, 244, 163, 15, 217, 206, 195, 174, 186, 121, 151, 32, 132, 233, 14, 173, 1, 234, 51, 9, 0, 0, 0, 0];
const GENESIS_BLOCK_HASH_SIGNET: [u8; 32] = [246, 30, 238, 59, 99, 163, 128, 164, 119, 160, 99, 175, 50, 178, 187, 201, 124, 159, 249, 240, 31, 44, 66, 37, 233, 115, 152, 129, 8, 0, 0, 0];
const GENESIS_BLOCK_HASH_REGTEST: [u8; 32] = [6, 34, 110, 70, 17, 26, 11, 89, 202, 175, 18, 96, 67, 235, 91, 191, 40, 195, 79, 58, 94, 51, 42, 31, 199, 178, 183, 60, 241, 136, 145, 15];

/// The uniquely identifying hash of the target blockchain.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChainHash([u8; 32]);
impl_array_newtype!(ChainHash, u8, 32);
impl_bytes_newtype!(ChainHash, 32);

impl ChainHash {
    /// Returns the hash of the `network` genesis block for use as a chain hash.
    ///
    /// See [BOLT 0](https://github.com/lightning/bolts/blob/ffeece3dab1c52efdb9b53ae476539320fa44938/00-introduction.md#chain_hash)
    /// for specification.
    pub fn using_genesis_block(network: Network) -> Self {
        match network {
            Network::Bitcoin => ChainHash(GENESIS_BLOCK_HASH_BITCOIN),
            Network::Testnet => ChainHash(GENESIS_BLOCK_HASH_TESTNET),
            Network::Signet => ChainHash(GENESIS_BLOCK_HASH_SIGNET),
            Network::Regtest => ChainHash(GENESIS_BLOCK_HASH_REGTEST),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::hashes::hex::{ToHex, FromHex};
    use crate::network::constants::Network;
    use crate::consensus::encode::serialize;
    use crate::blockdata::locktime::PackedLockTime;

    #[test]
    fn bitcoin_genesis_first_transaction() {
        let gen = pepecoin_genesis_tx();

        assert_eq!(gen.version, 1);
        assert_eq!(gen.input.len(), 1);
        assert_eq!(gen.input[0].previous_output.txid, Hash::all_zeros());
        assert_eq!(gen.input[0].previous_output.vout, 0xFFFFFFFF);
        assert_eq!(serialize(&gen.input[0].script_sig),
                   Vec::from_hex("5104ffff001d01044957534a20312f32322f3234202d204665642052657669657720436c656172732043656e7472616c2042616e6b204f6666696369616c73206f662056696f6c6174696e672052756c6573").unwrap());

        assert_eq!(gen.input[0].sequence, Sequence::MAX);
        assert_eq!(gen.output.len(), 1);
        assert_eq!(serialize(&gen.output[0].script_pubkey),
                   Vec::from_hex("43410436d04f40a76a1094ea10b14a513b62bfd0b47472dda1c25aa9cf8266e53f3c4353680146177f8a3b328ed2c6e02f2b8e051d9d5ffc61a4e6ccabd03409109a5aac").unwrap());
        assert_eq!(gen.output[0].value, 88 * COIN_VALUE);
        assert_eq!(gen.lock_time, PackedLockTime::ZERO);

        assert_eq!(gen.wtxid().to_hex(), "d22a1ba59a39cbd5904624933efb822c8baa121f97060c4cc9ea2f00a4bc6512");
    }

    #[test]
    fn bitcoin_genesis_full_block() {
        let gen = genesis_block(Network::Bitcoin);

        assert_eq!(gen.header.version, 1);
        assert_eq!(gen.header.prev_blockhash, Hash::all_zeros());
        assert_eq!(gen.header.merkle_root.to_hex(), "d22a1ba59a39cbd5904624933efb822c8baa121f97060c4cc9ea2f00a4bc6512");

        assert_eq!(gen.header.time, 1705975200);
        assert_eq!(gen.header.bits, 0x1e0ffff0);
        assert_eq!(gen.header.nonce, 427444);
        assert_eq!(gen.header.block_hash().to_hex(), "37981c0c48b8d48965376c8a42ece9a0838daadb93ff975cb091f57f8c2a5faa");
    }


    #[test]
    fn testnet_genesis_full_block() {
        let gen = genesis_block(Network::Testnet);
        assert_eq!(gen.header.version, 1);
        assert_eq!(gen.header.prev_blockhash, Hash::all_zeros());
        assert_eq!(gen.header.merkle_root.to_hex(), "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b");
        assert_eq!(gen.header.time, 1296688602);
        assert_eq!(gen.header.bits, 0x1d00ffff);
        assert_eq!(gen.header.nonce, 414098458);
        assert_eq!(gen.header.block_hash().to_hex(), "000000000933ea01ad0ee984209779baaec3ced90fa3f408719526f8d77f4943");
    }

    #[test]
    fn signet_genesis_full_block() {
        let gen = genesis_block(Network::Signet);
        assert_eq!(gen.header.version, 1);
        assert_eq!(gen.header.prev_blockhash, Hash::all_zeros());
        assert_eq!(gen.header.merkle_root.to_hex(), "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b");
        assert_eq!(gen.header.time, 1598918400);
        assert_eq!(gen.header.bits, 0x1e0377ae);
        assert_eq!(gen.header.nonce, 52613770);
        assert_eq!(gen.header.block_hash().to_hex(), "00000008819873e925422c1ff0f99f7cc9bbb232af63a077a480a3633bee1ef6");
    }

    // The *_chain_hash tests are sanity/regression tests, they verify that the const byte array
    // representing the genesis block is the same as that created by hashing the genesis block.
    fn chain_hash_and_genesis_block(network: Network) {
        use crate::hashes::sha256;

        if network == Network::Bitcoin {
            // For Pepecoin, GENESIS_BLOCK_HASH_BITCOIN is the Scrypt hash,
            // while block_hash() returns the Sha256d hash. Skip this check.
            return;
        }

        // The genesis block hash is a double-sha256 and it is displayed backwards.
        let genesis_hash = genesis_block(network).block_hash();
        // We abuse the sha256 hash here so we get a LowerHex impl that does not print the hex backwards.
        let hash = sha256::Hash::from_slice(&genesis_hash.into_inner()).unwrap();
        let want = format!("{:02x}", hash);

        let chain_hash = ChainHash::using_genesis_block(network);
        let got = format!("{:02x}", chain_hash);

        // Compare strings because the spec specifically states how the chain hash must encode to hex.
        assert_eq!(got, want);
    }

    macro_rules! chain_hash_genesis_block {
        ($($test_name:ident, $network:expr);* $(;)*) => {
            $(
                #[test]
                fn $test_name() {
                    chain_hash_and_genesis_block($network);
                }
            )*
        }
    }

    chain_hash_genesis_block! {
        mainnet_chain_hash_genesis_block, Network::Bitcoin;
        testnet_chain_hash_genesis_block, Network::Testnet;
        signet_chain_hash_genesis_block, Network::Signet;
        regtest_chain_hash_genesis_block, Network::Regtest;
    }

    // Test vector taken from: https://github.com/lightning/bolts/blob/master/00-introduction.md
    #[test]
    fn mainnet_chain_hash_test_vector() {
        let got = ChainHash::using_genesis_block(Network::Bitcoin).to_hex();
        let want = "aa5f2a8c7ff591b05c97ff93dbaa8d83a0e9ec428a6c376589d4b8480c1c9837";
        assert_eq!(got, want);
    }
}

