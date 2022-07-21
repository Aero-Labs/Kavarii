elrond_wasm::imports!();
elrond_wasm::derive_imports!();

// All storage mods for contract // referenced from lib.rs
#[elrond_wasm::module]
pub trait Storage{

    fn get_current_time(&self) -> u64 {
        self.blockchain().get_block_timestamp()
    }

    #[view(pushSubscriber)]
    #[storage_mapper("pushSubscriber")]
    fn subscribe_list(&self, owner: &ManagedAddress) -> LinkedListMapper<ManagedBuffer>;

    #[view(pushUnSubscribe)]
    #[storage_mapper("pushUnSubscribe")]
    fn unsubscribe_list(&self, owner: &ManagedAddress) -> LinkedListMapper<ManagedBuffer>;

    #[view(getDislikes)]
    #[storage_mapper("getDislikes")]
    fn submit_dislike(&self, identifier: &ManagedBuffer) -> LinkedListMapper<ManagedAddress>;

    #[view(getReports)]
    #[storage_mapper("getReports")]
    fn submit_report(&self, identifier: &ManagedBuffer) -> LinkedListMapper<ManagedAddress>;

    #[view(getLikes)]
    #[storage_mapper("getLikes")]
    fn submit_like(&self, identifier: &ManagedBuffer) -> LinkedListMapper<ManagedAddress>;

    #[view(getTips)]
    #[storage_mapper("getTips")]
    fn send_tip(&self, address: &ManagedAddress, identifier: &ManagedBuffer) -> LinkedListMapper<BigUint>;

    #[view(getValidToken)]
    #[storage_mapper("tokenName")]
    fn cf_token_name(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("contractOwner")]
    fn contract_owner(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(lastDrip)]
    #[storage_mapper("lastDrip")]
    fn last_drip(&self, caller: &ManagedAddress) -> SingleValueMapper<u64>;

    #[view(depositTokens)]
    #[storage_mapper("depositTokens")]
    fn deposit_tokens(&self, caller: &ManagedAddress) -> SingleValueMapper<BigUint>;

    #[view(faucetAmount)]
    #[storage_mapper("faucetAmount")]
    fn faucet_amount(&self) -> SingleValueMapper<BigUint>;

    #[view(getAccount)]
    #[storage_mapper("getAccount")]
    fn new_account(&self, address: &ManagedAddress) -> SingleValueMapper<ManagedBuffer>;

    
}

