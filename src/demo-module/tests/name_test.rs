use demo_module::{ BankA, CallMessage, BankConfig};
use sov_modules_api::utils::generate_address;
use sov_modules_api::{Context, Module};
use sov_state::{ProverStorage, WorkingSet};
use sov_modules_api::default_context::DefaultContext;
mod helpers;


#[test]
fn initial_name() {
  let bank_config: BankConfig<DefaultContext> = BankConfig {tokens: vec![] };
  let tmpdir = tempfile::tempdir().unwrap();
  let mut working_set = WorkingSet::new(ProverStorage::with_path(tmpdir.path()).unwrap());
  
  let bank: BankA<DefaultContext> = BankA::default();
  bank.genesis(&bank_config, &mut working_set).unwrap();

  let name_opt = bank.get_name_of(&mut working_set);
  assert_eq!("test_str".to_owned(), name_opt.unwrap());

  
  let sender_address = generate_address::<DefaultContext>("sender");
  let sender_context = DefaultContext::new(sender_address);
  
  let update_name_message = CallMessage::UpdateName::<DefaultContext> {
    name: "test_str1".to_owned()
  };

  bank.call(update_name_message, &sender_context, &mut working_set).expect("Failed to create token");

  let name_opt = bank.get_name_of(&mut working_set);
  assert_eq!("test_str1".to_owned(), name_opt.unwrap());

}
