elrond_wasm::imports!();
elrond_wasm::derive_imports!();

// All storage mods for contract // referenced from lib.rs
#[elrond_wasm::module]
pub trait Functions: crate::storage::Storage{

    // create new account
    #[endpoint(createAccount)]
    fn create_account(
        &self,
        account: ManagedBuffer,
    ){
        let caller = self.blockchain().get_caller();
        self.new_account(&caller).set(account);
    }


    // Like function
    #[endpoint(createLike)]
    #[payable("*")]
    fn create_like(
        &self,
        identifier: ManagedBuffer,
        owner: ManagedAddress,
        #[payment_token] token: TokenIdentifier,
        #[payment] payment: BigUint
        ){
    
            // Require them to use ATIP
            require!(token == self.cf_token_name().get(), "wrong token");
    
            // get callers address
            let caller = self.blockchain().get_caller();
    
            // store the like so we can track it in a contract call
            self.submit_like(&identifier).push_back(caller);
    
            // divert funds to the NFT owner
            self.send().direct(&owner, &token, 0, &BigUint::from(payment), &[]);
        }

    // Dislike function
    #[endpoint(createDislike)]
    #[payable("*")]
    fn create_dislike(
         &self,
         identifier: ManagedBuffer,
         owner: ManagedAddress,
         #[payment_token] token: TokenIdentifier,
         #[payment] payment: BigUint

    ) {

        // Require them to use ATIP
        require!(token == self.cf_token_name().get(), "wrong token");

        // get callers address
        let caller = self.blockchain().get_caller();

        let _hold = owner;

        // store the dislike so we can track it in a contract call
        self.submit_dislike(&identifier).push_back(caller);
        // get contract pool address
        let pool = self.contract_owner().get();

        // divert funds to the contract staking pool
        self.send().direct(&pool, &token, 0, &BigUint::from(payment), &[]);
    }
 

    #[endpoint(subscribe)]
    fn subscribe_channel(
        &self,
        channel: ManagedAddress,
        caller: ManagedBuffer,
    ){
        self.subscribe_list(&channel).push_back(caller);
    }

    #[endpoint(unSubscribe)]
    fn unsubscribe_channel(
        &self,
        channel: ManagedAddress,
        caller: ManagedBuffer,
    ){
        self.unsubscribe_list(&channel).push_back(caller);
    }

    // Report NFT function
    #[endpoint(report)]
    #[payable("*")]
    fn report_nft(
        &self,
        identifier: ManagedBuffer,
        owner: ManagedAddress,
        #[payment_token] token: TokenIdentifier,
        #[payment] payment: BigUint
    ){
 
        // Require them to use ATIP
        require!(token == self.cf_token_name().get(), "wrong token");
 
        // get callers address
        let caller = self.blockchain().get_caller();
 
        // store the dislike so we can track it in a contract call
        self.submit_report(&identifier).push_back(caller);

        let _hold = owner;
 
        // get contract pool address
        let pool = self.contract_owner().get();
 
        // divert funds to the contract staking pool
        self.send().direct(&pool, &token, 0, &BigUint::from(payment), &[]);
     }



    // Sent creator a tip
    #[endpoint(sendTip)]
    #[payable("*")]
    fn send_tips(
        &self,
        identifier: ManagedBuffer,
        owner: ManagedAddress,
        #[payment_token] token: TokenIdentifier,
        #[payment] payment: BigUint
    ) {
  
        // Require them to use ATIP
        require!(token == self.cf_token_name().get(), "wrong token");
  
        // get callers address
        let caller = self.blockchain().get_caller();
  
        // create a clone of the payment amount to store in a view function
        let clone_payment =  payment.clone();
  
        // store the dislike so we can track it in a contract call
        self.send_tip(&caller, &identifier).push_back(clone_payment);
  
        // divert funds to the NFT owner
        self.send().direct(&owner, &token, 0, &BigUint::from(payment), &[]);
    }
  

    #[endpoint(maxBurn)]
    #[payable("*")]
    fn local_burn(&self, #[payment_token] token_identifier: TokenIdentifier, #[payment] amount: BigUint, #[payment_nonce] nonce: u64) {
        self.send().esdt_local_burn(&token_identifier, nonce, &amount);
    }

    #[endpoint(faucetDrip)]
    #[allow(arithmetic_overflow)]
    fn faucet_drip(&self){
        let caller = self.blockchain().get_caller();
        let last_drip = self.last_drip(&caller).get();
        let current_time = self.blockchain().get_block_timestamp();

        let token = self.cf_token_name().get();

        let drip_diff = current_time - 86400u64;

        require!(last_drip > drip_diff, "You can only get one drip a day");

        let drip = self.faucet_amount().get();

        self.send().direct(&caller, &token, 0, &BigUint::from(drip), &[]);
    }


    #[only_owner]
    #[endpoint(deposit)]
    #[payable("*")]
    fn deposit(&self, #[payment_token] token_identifier: TokenIdentifier, #[payment] amount: BigUint){

        let token = self.cf_token_name().get();

        require!(token_identifier == token, "wrong token to deposit");

        let caller = self.blockchain().get_caller();

        let pool_size = self.deposit_tokens(&caller).get();

        let new_funds = amount + pool_size;

        self.deposit_tokens(&caller).set(new_funds);
    }

    #[only_owner]
    #[endpoint(setFaucet)]
    fn set_faucet(&self, amount: BigUint){
        self.faucet_amount().set(amount);
    }

}
