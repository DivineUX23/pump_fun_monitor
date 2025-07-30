//! Unit tests for WebSocket server filtering functionality.


use super::*;
use crate::data_models::{PumpFunData, TokenDetails};
use chrono::Utc;

/// function to create a dummy token creation event for testing.
fn create_test_event(creator: &str, name: &str, symbol: &str) -> TokenCreatedEvent {
    TokenCreatedEvent {
        event_type: "tokenCreated".to_string(),
        timestamp: Utc::now(),
        transaction_signature: "test_sig_123456789".to_string(),
        token: TokenDetails {
            mint_address: "test_mint_ABC123def456".to_string(),
            name: name.to_string(),
            symbol: symbol.to_string(),
            uri: "https://test.example.com/metadata.json".to_string(),
            creator: creator.to_string(),
            supply: 1_000_000,
            decimals: 6,
        },
        pump_data: PumpFunData {
            bonding_curve: "test_curve_GHI789jkl012".to_string(),
            virtual_sol_reserves: 30_000_000_000,
            virtual_token_reserves: 1_073_000_000_000_000,
        },
    }
}

#[test]
fn test_no_filter_matches_all() {
    let event = create_test_event("creator_A", "My Token", "TKN");
    let filter = FilterCriteria::default();
    assert!(matches_filter(&event, &filter));
}

#[test]
fn test_filter_by_creator_exact_match() {
    let event = create_test_event("creator_A", "My Token", "TKN");
    let filter = FilterCriteria {
        creator: Some("creator_A".to_string()),
        ..Default::default()
    };
    assert!(matches_filter(&event, &filter));
}

#[test]
fn test_filter_by_creator_no_match() {
    let event = create_test_event("creator_A", "My Token", "TKN");
    let filter = FilterCriteria {
        creator: Some("creator_B".to_string()),
        ..Default::default()
    };
    assert!(!matches_filter(&event, &filter));
}

#[test]
fn test_filter_by_symbol_case_insensitive_match() {
    let event = create_test_event("creator_A", "My Token", "TKN");
    
    // test lowercase filter against uppercase symbol
    let filter_lower = FilterCriteria {
        symbol: Some("tkn".to_string()),
        ..Default::default()
    };
    
    // test uppercase filter against uppercase symbol
    let filter_upper = FilterCriteria {
        symbol: Some("TKN".to_string()),
        ..Default::default()
    };
    
    assert!(matches_filter(&event, &filter_lower));
    assert!(matches_filter(&event, &filter_upper));
}

#[test]
fn test_filter_by_symbol_no_match() {
    let event = create_test_event("creator_A", "My Token", "TKN");
    let filter = FilterCriteria {
        symbol: Some("DOGE".to_string()),
        ..Default::default()
    };
    assert!(!matches_filter(&event, &filter));
}

#[test]
fn test_filter_by_name_contains_case_insensitive_match() {
    let event = create_test_event("creator_A", "My Awesome Token", "TKN");
    
    // test various case combinations
    let filters = vec![
        "Awesome",
        "awesome", 
        "AWESOME",
        "Token",
        "token",
        "My",
        "my"
    ];
    
    for filter_text in filters {
        let filter = FilterCriteria {
            name_contains: Some(filter_text.to_string()),
            ..Default::default()
        };
        assert!(matches_filter(&event, &filter), "Failed to match '{}'", filter_text);
    }
}

#[test]
fn test_filter_by_name_contains_no_match() {
    let event = create_test_event("creator_A", "My Awesome Token", "TKN");
    let filter = FilterCriteria {
        name_contains: Some("Boring".to_string()),
        ..Default::default()
    };
    assert!(!matches_filter(&event, &filter));
}

#[test]
fn test_filter_by_multiple_criteria_all_match() {
    let event = create_test_event("creator_A", "My Awesome Token", "TKN");
    let filter = FilterCriteria {
        creator: Some("creator_A".to_string()),
        symbol: Some("TKN".to_string()),
        name_contains: Some("Awesome".to_string()),
    };
    assert!(matches_filter(&event, &filter));
}

#[test]
fn test_filter_by_multiple_criteria_partial_match_fails() {
    let event = create_test_event("creator_A", "My Token", "TKN");
    
    // creator matches but symbol doesn't
    let filter1 = FilterCriteria {
        creator: Some("creator_A".to_string()),
        symbol: Some("FAIL".to_string()),
        ..Default::default()
    };
    assert!(!matches_filter(&event, &filter1));
    
    // symbol matches but creator doesn't
    let filter2 = FilterCriteria {
        creator: Some("creator_B".to_string()),
        symbol: Some("TKN".to_string()),
        ..Default::default()
    };
    assert!(!matches_filter(&event, &filter2));
}

#[test]
fn test_filter_edge_cases() {
    let event = create_test_event("", "Token", "");
    
    let filter_empty_creator = FilterCriteria {
        creator: Some("".to_string()),
        ..Default::default()
    };
    assert!(matches_filter(&event, &filter_empty_creator));
    
    let filter_empty_symbol = FilterCriteria {
        symbol: Some("".to_string()),
        ..Default::default()
    };
    assert!(matches_filter(&event, &filter_empty_symbol));
    
    let filter_empty_name = FilterCriteria {
        name_contains: Some("".to_string()),
        ..Default::default()
    };
    assert!(matches_filter(&event, &filter_empty_name));
}

#[test]
fn test_filter_real_world_scenarios() {
    // Test realistic pump.fun token scenarios
    let doge_token = create_test_event(
        "DEF456ghi789JKL012mno345PQR678stu901VWX234yza567BCD890efg123",
        "DogeToTheMoon",
        "DOGE"
    );
    
    let pepe_token = create_test_event(
        "ABC123def456GHI789jkl012MNO345pqr678STU901vwx234YZA567bcd890",
        "PepeCoin Classic",
        "PEPE"
    );
    
    // filter for DOGE tokens
    let doge_filter = FilterCriteria {
        symbol: Some("DOGE".to_string()),
        ..Default::default()
    };
    assert!(matches_filter(&doge_token, &doge_filter));
    assert!(!matches_filter(&pepe_token, &doge_filter));
    
    // filter for tokens with "moon" in name
    let moon_filter = FilterCriteria {
        name_contains: Some("moon".to_string()),
        ..Default::default()
    };
    assert!(matches_filter(&doge_token, &moon_filter));
    assert!(!matches_filter(&pepe_token, &moon_filter));
    
    // filter for specific creator
    let creator_filter = FilterCriteria {
        creator: Some("DEF456ghi789JKL012mno345PQR678stu901VWX234yza567BCD890efg123".to_string()),
        ..Default::default()
    };
    assert!(matches_filter(&doge_token, &creator_filter));
    assert!(!matches_filter(&pepe_token, &creator_filter));
}
