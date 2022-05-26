#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

mod nft_module;

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct ExampleAttributes {
    pub creation_timestamp: u64,
}

#[elrond_wasm::contract]
pub trait NftMinter: nft_module::NftModule {
    #[init]
    fn init(&self,token_name: TokenIdentifier) {
        require!(token_name.is_egld() || token_name.is_valid_esdt_identifier(),"Invalid token provided");
        let contract_owner = &self.blockchain().get_caller();
        self.cf_token_name().set(&token_name);
        self.contract_owner().set(contract_owner);
    }

    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::redundant_closure)]

    #[endpoint(createNft)]
    fn create_nft(
        &self,
        nft_token_id: TokenIdentifier,
        name: ManagedBuffer,
        royalties: BigUint,
        attributes: ManagedBuffer,
        #[var_args] uri: MultiValueEncoded<ManagedBuffer>,
    ) {


        self.create_nft_with_attributes(
            nft_token_id,
            name,
            royalties,
            attributes,
            uri,
        );
    }



    // create new account
    #[endpoint(createAccount)]
    fn create_account(
        &self,
        account: ManagedBuffer,
    ){
        let caller = self.blockchain().get_caller();
        self.new_account(&caller).set(account);
    }

    #[view(getAccount)]
    #[storage_mapper("getAccount")]
    fn new_account(&self, address: &ManagedAddress) -> SingleValueMapper<ManagedBuffer>;






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

}

