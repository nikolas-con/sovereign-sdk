use sov_election::ElectionConfig;
pub use sov_modules_api::default_context::DefaultContext;
use sov_modules_api::default_signature::private_key::DefaultPrivateKey;
use sov_modules_api::utils::generate_address;
use sov_modules_api::{Context, PrivateKey, PublicKey};
pub use sov_state::config::Config as StorageConfig;
use sov_value_setter::ValueSetterConfig;

/// Creates config for a rollup with some default settings, the config is used in demos and tests.
use crate::runtime::GenesisConfig;

pub const DEMO_SEQUENCER_DA_ADDRESS: [u8; 32] = [1; 32];
pub const LOCKED_AMOUNT: u64 = 50;
pub const DEMO_SEQ_PUB_KEY_STR: &str = "seq_pub_key";
pub const DEMO_TOKEN_NAME: &str = "sov-demo-token";

pub fn create_demo_genesis_config<C: Context>(
    initial_sequencer_balance: u64,
    sequencer_address: C::Address,
    sequencer_da_address: Vec<u8>,
    value_setter_admin_private_key: &DefaultPrivateKey,
    election_admin_private_key: &DefaultPrivateKey,
) -> GenesisConfig<C> {
    let token_config: sov_bank::TokenConfig<C> = sov_bank::TokenConfig {
        token_name: DEMO_TOKEN_NAME.to_owned(),
        address_and_balances: vec![(sequencer_address.clone(), initial_sequencer_balance)],
        authorized_minters: vec![sequencer_address.clone()],
        salt: 0,
    };
    
    let bank_config = sov_bank::BankConfig {
        tokens: vec![token_config],
    };

    let demo_module_config = demo_module::ModuleConfig {};

    let token_address = sov_bank::get_genesis_token_address::<C>(
        &bank_config.tokens[0].token_name,
        bank_config.tokens[0].salt,
    );

    let sequencer_registry_config = sov_sequencer_registry::SequencerConfig {
        seq_rollup_address: sequencer_address,
        seq_da_address: sequencer_da_address,
        coins_to_lock: sov_bank::Coins {
            amount: LOCKED_AMOUNT,
            token_address,
        },
        preferred_sequencer: None,
    };

    let value_setter_config = ValueSetterConfig {
        admin: value_setter_admin_private_key.pub_key().to_address(),
    };

    let election_config = ElectionConfig {
        admin: election_admin_private_key.pub_key().to_address(),
    };

    GenesisConfig::new(
        bank_config,
        demo_module_config,
        sequencer_registry_config,
        (),
        election_config,
        value_setter_config,
        sov_accounts::AccountConfig { pub_keys: vec![] }
    )
}

pub fn create_demo_config(
    initial_sequencer_balance: u64,
    value_setter_admin_private_key: &DefaultPrivateKey,
    election_admin_private_key: &DefaultPrivateKey,
) -> GenesisConfig<DefaultContext> {
    create_demo_genesis_config::<DefaultContext>(
        initial_sequencer_balance,
        generate_address::<DefaultContext>(DEMO_SEQ_PUB_KEY_STR),
        DEMO_SEQUENCER_DA_ADDRESS.to_vec(),
        value_setter_admin_private_key,
        election_admin_private_key,
    )
}
