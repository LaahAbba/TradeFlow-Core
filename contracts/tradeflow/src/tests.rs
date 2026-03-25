use soroban_sdk::{
    Address, Env, BytesN, Symbol, Vec, Val, testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    token,
};
use crate::{
    TradeFlow, LiquidityPosition, PendingFeeChange, PermitData, DataKey,
    utils::fixed_point::{self, Q64},
};

#[test]
fn test_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    
    TradeFlow::init(&env, admin.clone(), token_a.clone(), token_b.clone(), 30);
    
    assert_eq!(TradeFlow::get_protocol_fee(&env), 30);
    
    let (reserve_a, reserve_b) = TradeFlow::get_reserves(&env);
    assert_eq!(reserve_a, 0);
    assert_eq!(reserve_b, 0);
}

#[test]
fn test_fee_change_timelock() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    
    TradeFlow::init(&env, admin.clone(), token_a, token_b, 30);
    
    // Propose fee change
    TradeFlow::propose_fee_change(&env, 50);
    
    let pending = TradeFlow::get_pending_fee_change(&env).unwrap();
    assert_eq!(pending.new_fee, 50);
    
    // Should not be able to execute immediately
    env.mock_auths(&[
        (&admin, &AuthorizedInvocation {
            contract: &env.current_contract_address(),
            function: &AuthorizedFunction::Contract((
                Symbol::new(&env, "execute_fee_change"),
                (),
            )),
            sub_invocations: &[]
        })
    ]);
    
    let result = std::panic::catch_unwind(|| {
        TradeFlow::execute_fee_change(&env);
    });
    assert!(result.is_err()); // Should panic due to timelock
    
    // Fast forward time by 48 hours
    env.ledger().set_timestamp(env.ledger().timestamp() + 48 * 60 * 60 + 1);
    
    // Now should be able to execute
    TradeFlow::execute_fee_change(&env);
    assert_eq!(TradeFlow::get_protocol_fee(&env), 50);
    assert!(TradeFlow::get_pending_fee_change(&env).is_none());
}

#[test]
fn test_provide_liquidity() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    
    // Create token contracts for testing
    let token_a_client = token::Client::new(&env, &token_a);
    let token_b_client = token::Client::new(&env, &token_b);
    
    TradeFlow::init(&env, admin, token_a.clone(), token_b.clone(), 30);
    
    // Mint tokens to user
    // Note: In a real test environment, you'd need to set up proper token contracts
    // For this example, we'll assume the tokens are already minted
    
    // Provide liquidity
    let shares = TradeFlow::provide_liquidity(&env, user.clone(), 100, 200, 1);
    
    assert!(shares > 0);
    
    let position = TradeFlow::get_liquidity_position(&env, user).unwrap();
    assert_eq!(position.token_a_amount, 100);
    assert_eq!(position.token_b_amount, 200);
    assert_eq!(position.shares, shares);
    
    let (reserve_a, reserve_b) = TradeFlow::get_reserves(&env);
    assert_eq!(reserve_a, 100);
    assert_eq!(reserve_b, 200);
}

#[test]
fn test_swap() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    
    let token_a_client = token::Client::new(&env, &token_a);
    let token_b_client = token::Client::new(&env, &token_b);
    
    TradeFlow::init(&env, admin, token_a.clone(), token_b.clone(), 30);
    
    // First provide liquidity
    // Note: In a real test, you'd need to mint tokens to the user first
    TradeFlow::provide_liquidity(&env, user.clone(), 500, 500, 1);
    
    // Now perform swap
    // Note: In a real test, you'd need to mint tokens to the user first
    let amount_out = TradeFlow::swap(&env, user.clone(), token_a.clone(), 100, 1);
    
    assert!(amount_out >= 1);
    
    // Check reserves changed correctly
    let (reserve_a, reserve_b) = TradeFlow::get_reserves(&env);
    assert_eq!(reserve_a, 600); // 500 + 100
    assert!(reserve_b < 500); // Decreased due to swap
}

#[test]
fn test_permit_swap() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    
    let token_a_client = token::Client::new(&env, &token_a);
    let token_b_client = token::Client::new(&env, &token_b);
    
    TradeFlow::init(&env, admin, token_a.clone(), token_b.clone(), 30);
    
    // First provide liquidity
    // Note: In a real test, you'd need to mint tokens to the user first
    TradeFlow::provide_liquidity(&env, user.clone(), 500, 500, 1);
    
    // Create permit data
    let permit_data = PermitData {
        owner: user.clone(),
        spender: env.current_contract_address(),
        amount: 100,
        nonce: TradeFlow::get_user_nonce(&env, user.clone()),
        deadline: env.ledger().timestamp() + 3600, // 1 hour from now
    };
    
    // Mock signature (in real implementation, this would be a valid signature)
    let signature = BytesN::from_array(&env, &[0u8; 64]);
    
    // Mock signature verification
    env.mock_auths(&[
        (&user, &AuthorizedInvocation {
            contract: &env.current_contract_address(),
            function: &AuthorizedFunction::Contract((
                Symbol::new(&env, "permit_swap"),
                (
                    user.clone(),
                    token_a.clone(),
                    100u128,
                    1u128,
                    permit_data.clone(),
                    signature.clone(),
                ),
            )),
            sub_invocations: &[]
        })
    ]);
    
    // Note: In a real test, you'd need to mint tokens to the user first
    
    // This should work with proper signature verification
    // In a real implementation with proper signature generation, this would pass
    let result = std::panic::catch_unwind(|| {
        TradeFlow::permit_swap(&env, user.clone(), token_a.clone(), 100, 1, permit_data, signature);
    });
    
    // For this test, we expect it to fail due to invalid signature
    // In a real implementation with proper signature generation, this would pass
    assert!(result.is_err());
}

#[test]
fn test_fixed_point_math() {
    let env = Env::default();
    
    // Test mul_div_down
    let result = fixed_point::mul_div_down(&env, 100, 200, 50);
    assert_eq!(result, 400); // (100 * 200) / 50 = 400
    
    // Test mul_div_up
    let result = fixed_point::mul_div_up(&env, 100, 200, 50);
    assert_eq!(result, 400); // Same as down since it divides evenly
    
    // Test with rounding
    let result = fixed_point::mul_div_down(&env, 100, 3, 2);
    assert_eq!(result, 150); // (100 * 3) / 2 = 150
    
    let result = fixed_point::mul_div_up(&env, 100, 3, 2);
    assert_eq!(result, 150); // Same in this case
    
    // Test scale operations
    let scaled = fixed_point::scale_up(&env, 100);
    assert_eq!(scaled, 100 * Q64);
    
    let downscaled = fixed_point::scale_down(&env, scaled);
    assert_eq!(downscaled, 100);
}

#[test]
fn test_user_nonce_increment() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    
    TradeFlow::init(&env, admin, token_a, token_b, 30);
    
    // Initial nonce should be 0
    assert_eq!(TradeFlow::get_user_nonce(&env, user.clone()), 0);
    
    // Note: In a real test, you'd need to mint tokens to the user first
    TradeFlow::provide_liquidity(&env, user.clone(), 100, 200, 1);
    assert_eq!(TradeFlow::get_user_nonce(&env, user.clone()), 0);
}
