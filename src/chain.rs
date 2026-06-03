use {super::*, bitcoin::Amount, clap::ValueEnum};

#[derive(Default, ValueEnum, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Chain {
  #[default]
  #[clap(alias("main"))]
  Mainnet,
  #[clap(alias("test"))]
  Testnet,
  Signet,
  Regtest,
}

impl FromStr for Chain {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "mainnet" | "main" => Ok(Self::Mainnet),
      "testnet" | "test" => Ok(Self::Testnet),
      "signet" => Ok(Self::Signet),
      "regtest" => Ok(Self::Regtest),
      _ => bail!("unknown chain: {s}"),
    }
  }
}

impl Chain {
  pub(crate) fn network(self) -> Network {
    match self {
      Self::Mainnet => Network::Bitcoin,
      Self::Testnet => Network::Testnet,
      Self::Signet => Network::Signet,
      Self::Regtest => Network::Regtest,
    }
  }

  pub(crate) fn default_rpc_port(self) -> u16 {
    match self {
      Self::Mainnet => 20760,
      Self::Regtest => 18332,
      Self::Signet => 38332,
      Self::Testnet => 20761,
    }
  }

  pub(crate) fn default_fee_rate(self) -> FeeRate {
    match self {
      Self::Mainnet | Self::Regtest | Self::Signet | Self::Testnet => {
        FeeRate::try_from(10000.0).unwrap()
      }
    }
  }

  pub(crate) fn min_fee_rate(self) -> FeeRate {
    match self {
      Self::Mainnet | Self::Regtest | Self::Signet | Self::Testnet => {
        FeeRate::try_from(10000.0).unwrap()
      }
    }
  }

  pub(crate) fn default_postage(self) -> Amount {
    match self {
      Self::Mainnet | Self::Regtest | Self::Signet | Self::Testnet => Amount::from_sat(100_000),
    }
  }

  pub(crate) fn inscription_content_size_limit(self) -> Option<usize> {
    match self {
      Self::Mainnet | Self::Regtest => None,
      Self::Testnet => None,
      Self::Signet => Some(1024),
    }
  }

  pub(crate) fn first_inscription_height(self) -> u32 {
    match self {
      // Wojakinals launch height; override via --first-inscription-height or ordwoj.yaml
      Self::Mainnet => 1,
      Self::Regtest => 0,
      Self::Signet => 0,
      Self::Testnet => 0,
    }
  }

  pub(crate) fn genesis_block(self) -> Block {
    let genesis_hex: &str = "0100000000000000000000000000000000000000000000000000000000000000000000001a798b6eef464db807d1aac4beb08a463cdd8863b202523cbfa0523225b8942d2a808259ffff001d0b87a14a0101000000010000000000000000000000000000000000000000000000000000000000000000ffffffff2404ffff001d01041c333832303137205072696365205068696c6c69702052657469726573ffffffff0100e40b5402000000434104678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac00000000";
    let genesis_buf: Vec<u8> = hex::decode(genesis_hex).unwrap();
    bitcoin::consensus::deserialize(&genesis_buf).unwrap()
  }

  pub(crate) fn address_from_script(
    self,
    script: &Script,
  ) -> Result<Address, bitcoin::util::address::Error> {
    Address::from_script(script, self.network())
  }

  pub(crate) fn join_with_data_dir(self, data_dir: &Path) -> PathBuf {
    match self {
      Self::Mainnet => data_dir.to_owned(),
      Self::Testnet => data_dir.join("testnet3"),
      Self::Signet => data_dir.join("signet"),
      Self::Regtest => data_dir.join("regtest"),
    }
  }
}

impl Display for Chain {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::Mainnet => "mainnet",
        Self::Regtest => "regtest",
        Self::Signet => "signet",
        Self::Testnet => "testnet",
      }
    )
  }
}
