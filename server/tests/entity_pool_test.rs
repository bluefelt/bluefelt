//! Tests for the entity pooling system

use bluefelt_core::engine::entity_pool::{EntityPool, EntityStorage};
use serde_json::json;

#[test]
fn test_entity_pool_basic_reuse() {
    let pool = EntityPool::new(10);
    let template = json!({"type": "card", "rank": "A", "suit": "hearts"});
    
    // First acquisition should allocate
    let entity1 = pool.acquire("card", &template);
    assert_eq!(entity1, template);
    assert_eq!(pool.stats().allocations, 1);
    assert_eq!(pool.stats().reuses, 0);
    
    // Release the entity back to pool
    pool.release("card", entity1);
    assert_eq!(pool.stats().returns, 1);
    
    // Next acquisition should reuse
    let entity2 = pool.acquire("card", &template);
    assert_eq!(entity2, template);
    assert_eq!(pool.stats().allocations, 1);
    assert_eq!(pool.stats().reuses, 1);
}

#[test]
fn test_entity_pool_template_reset() {
    let pool = EntityPool::new(10);
    let template1 = json!({"type": "card", "rank": "A", "suit": "hearts"});
    let template2 = json!({"type": "card", "rank": "K", "suit": "spades"});
    
    // Acquire and modify entity
    let mut entity = pool.acquire("card", &template1);
    if let Some(obj) = entity.as_object_mut() {
        obj.insert("modified".to_string(), json!(true));
    }
    
    // Release modified entity
    pool.release("card", entity);
    
    // Acquire with different template - should reset to new template
    let entity2 = pool.acquire("card", &template2);
    assert_eq!(entity2, template2);
    assert!(!entity2.get("modified").is_some());
}

#[test]
fn test_entity_pool_eviction() {
    let pool = EntityPool::new(2); // Small pool size
    
    let template = json!({"type": "card"});
    
    // Fill the pool
    pool.release("card", pool.acquire("card", &template));
    pool.release("card", pool.acquire("card", &template));
    
    assert_eq!(pool.stats().returns, 2);
    assert_eq!(pool.stats().evictions, 0);
    
    // Try to add one more - should evict
    pool.release("card", json!({"type": "card", "extra": true}));
    
    assert_eq!(pool.stats().returns, 2);
    assert_eq!(pool.stats().evictions, 1);
}

#[test]
fn test_entity_pool_multiple_types() {
    let pool = EntityPool::new(10);
    
    let card_template = json!({"type": "card"});
    let token_template = json!({"type": "token"});
    
    // Acquire different types
    let card = pool.acquire("card", &card_template);
    let token = pool.acquire("token", &token_template);
    
    assert_eq!(pool.stats().allocations, 2);
    
    // Release both
    pool.release("card", card);
    pool.release("token", token);
    
    // Reacquire - each should reuse from its own pool
    let _card2 = pool.acquire("card", &card_template);
    let _token2 = pool.acquire("token", &token_template);
    
    assert_eq!(pool.stats().allocations, 2);
    assert_eq!(pool.stats().reuses, 2);
}

#[test]
fn test_entity_storage_basic_operations() {
    let mut storage = EntityStorage::new();
    
    // Add entities
    storage.add_entity(
        "card1".to_string(),
        "card",
        json!({"rank": "A", "suit": "hearts"}),
        "deck"
    );
    
    storage.add_entity(
        "card2".to_string(),
        "card",
        json!({"rank": "K", "suit": "spades"}),
        "deck"
    );
    
    storage.add_entity(
        "card3".to_string(),
        "card",
        json!({"rank": "Q", "suit": "diamonds"}),
        "hand"
    );
    
    // Check counts
    assert_eq!(storage.total_entities(), 3);
    assert_eq!(storage.zone_entity_count("deck"), 2);
    assert_eq!(storage.zone_entity_count("hand"), 1);
    
    // Get zone entities
    let deck_entities = storage.get_zone_entities("deck");
    assert_eq!(deck_entities.len(), 2);
}

#[test]
fn test_entity_storage_movement() {
    let mut storage = EntityStorage::new();
    
    // Add entity
    storage.add_entity(
        "card1".to_string(),
        "card",
        json!({"rank": "A"}),
        "deck"
    );
    
    // Move entity
    storage.move_entity("card1", "deck", "hand").unwrap();
    
    assert_eq!(storage.zone_entity_count("deck"), 0);
    assert_eq!(storage.zone_entity_count("hand"), 1);
    
    // Try to move non-existent entity
    let result = storage.move_entity("card99", "deck", "hand");
    assert!(result.is_err());
}

#[test]
fn test_entity_storage_removal() {
    let mut storage = EntityStorage::new();
    
    // Add entities
    storage.add_entity(
        "card1".to_string(),
        "card",
        json!({"rank": "A"}),
        "deck"
    );
    
    storage.add_entity(
        "card2".to_string(),
        "card",
        json!({"rank": "K"}),
        "deck"
    );
    
    // Remove one
    let removed = storage.remove_entity("card1", "deck");
    assert!(removed.is_some());
    
    assert_eq!(storage.total_entities(), 1);
    assert_eq!(storage.zone_entity_count("deck"), 1);
}

#[test]
fn test_entity_storage_memory_tracking() {
    let mut storage = EntityStorage::new();
    
    // Add some entities with different sizes
    storage.add_entity(
        "small".to_string(),
        "token",
        json!({"type": "token"}),
        "zone1"
    );
    
    storage.add_entity(
        "large".to_string(),
        "complex",
        json!({
            "type": "complex",
            "data": "x".repeat(100),
            "nested": {
                "values": [1, 2, 3, 4, 5],
                "more": "data"
            }
        }),
        "zone2"
    );
    
    let usage = storage.memory_usage();
    assert!(usage > 100); // Should be more than 100 bytes with the large entity
}

#[test]
fn test_entity_pool_in_game_scenario() {
    // Simulate a card game scenario
    let pool = EntityPool::new(100);
    let mut storage = EntityStorage::new();
    
    // Standard 52-card deck
    let suits = ["hearts", "diamonds", "clubs", "spades"];
    let ranks = ["A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K"];
    
    // Create deck
    let mut card_id = 0;
    for suit in &suits {
        for rank in &ranks {
            let template = json!({
                "type": "card",
                "rank": rank,
                "suit": suit
            });
            
            let entity = pool.acquire("card", &template);
            storage.add_entity(
                format!("card_{}", card_id),
                "card",
                entity,
                "deck"
            );
            card_id += 1;
        }
    }
    
    assert_eq!(storage.total_entities(), 52);
    assert_eq!(pool.stats().allocations, 52);
    
    // Deal cards to players
    for i in 0..10 {
        storage.move_entity(&format!("card_{}", i), "deck", "player1_hand").unwrap();
    }
    for i in 10..20 {
        storage.move_entity(&format!("card_{}", i), "deck", "player2_hand").unwrap();
    }
    
    assert_eq!(storage.zone_entity_count("deck"), 32);
    assert_eq!(storage.zone_entity_count("player1_hand"), 10);
    assert_eq!(storage.zone_entity_count("player2_hand"), 10);
    
    // Discard and return cards to pool
    let discarded = storage.get_zone_entities("player1_hand");
    for (id, entity) in discarded {
        storage.remove_entity(&id, "player1_hand");
        pool.release("card", (*entity).clone());
    }
    
    assert_eq!(pool.stats().returns, 10);
    
    // Draw new cards - should reuse from pool
    for i in 0..5 {
        let template = json!({"type": "card", "rank": "A", "suit": "hearts"});
        let entity = pool.acquire("card", &template);
        storage.add_entity(
            format!("new_card_{}", i),
            "card",
            entity,
            "player1_hand"
        );
    }
    
    assert_eq!(pool.stats().reuses, 5);
    assert_eq!(pool.stats().allocations, 52); // No new allocations
}