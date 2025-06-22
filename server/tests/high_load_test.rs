//! High-load testing framework for server stress testing

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::{Semaphore, Mutex};
use tokio::time::{sleep, timeout};
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use serde_json::{json, Value};
use tokio::task::JoinSet;

/// Configuration for high-load tests
#[derive(Clone)]
struct LoadTestConfig {
    /// Number of concurrent users
    concurrent_users: usize,
    /// Number of lobbies to create
    lobby_count: usize,
    /// Games per lobby
    games_per_lobby: usize,
    /// Actions per game
    actions_per_game: usize,
    /// Message interval per user (milliseconds)
    message_interval_ms: u64,
    /// Test duration
    test_duration: Duration,
    /// Server URL
    server_url: String,
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self {
            concurrent_users: 100,
            lobby_count: 10,
            games_per_lobby: 5,
            actions_per_game: 20,
            message_interval_ms: 100,
            test_duration: Duration::from_secs(60),
            server_url: "ws://localhost:8000".to_string(),
        }
    }
}

/// Metrics collected during testing
#[derive(Default)]
struct LoadTestMetrics {
    /// Total connections attempted
    connections_attempted: AtomicU64,
    /// Successful connections
    connections_successful: AtomicU64,
    /// Failed connections
    connections_failed: AtomicU64,
    /// Messages sent
    messages_sent: AtomicU64,
    /// Messages received
    messages_received: AtomicU64,
    /// Errors encountered
    errors: AtomicU64,
    /// Games completed
    games_completed: AtomicU64,
    /// Average latency (microseconds)
    total_latency_us: AtomicU64,
    /// Latency samples
    latency_samples: AtomicU64,
}

impl LoadTestMetrics {
    fn record_latency(&self, latency: Duration) {
        self.total_latency_us.fetch_add(latency.as_micros() as u64, Ordering::Relaxed);
        self.latency_samples.fetch_add(1, Ordering::Relaxed);
    }

    fn average_latency_ms(&self) -> f64 {
        let samples = self.latency_samples.load(Ordering::Relaxed);
        if samples == 0 {
            return 0.0;
        }
        let total_us = self.total_latency_us.load(Ordering::Relaxed);
        (total_us as f64 / samples as f64) / 1000.0
    }

    fn print_summary(&self) {
        println!("\n========== High-Load Test Results ==========");
        println!("Connections:");
        println!("  Attempted: {}", self.connections_attempted.load(Ordering::Relaxed));
        println!("  Successful: {}", self.connections_successful.load(Ordering::Relaxed));
        println!("  Failed: {}", self.connections_failed.load(Ordering::Relaxed));
        println!("\nMessages:");
        println!("  Sent: {}", self.messages_sent.load(Ordering::Relaxed));
        println!("  Received: {}", self.messages_received.load(Ordering::Relaxed));
        println!("\nPerformance:");
        println!("  Average Latency: {:.2} ms", self.average_latency_ms());
        println!("  Games Completed: {}", self.games_completed.load(Ordering::Relaxed));
        println!("  Errors: {}", self.errors.load(Ordering::Relaxed));
        println!("==========================================\n");
    }
}

/// Simulated user for load testing
struct SimulatedUser {
    id: String,
    config: LoadTestConfig,
    metrics: Arc<LoadTestMetrics>,
    should_stop: Arc<AtomicBool>,
}

impl SimulatedUser {
    async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Connect to server
        self.metrics.connections_attempted.fetch_add(1, Ordering::Relaxed);
        
        let url = format!("{}/api/lobbies/test-{}/ws?player={}", 
            self.config.server_url, 
            self.id.split('-').next().unwrap_or("0"),
            self.id
        );
        
        let (ws_stream, _) = match timeout(Duration::from_secs(5), connect_async(&url)).await {
            Ok(Ok(stream)) => {
                self.metrics.connections_successful.fetch_add(1, Ordering::Relaxed);
                stream
            }
            _ => {
                self.metrics.connections_failed.fetch_add(1, Ordering::Relaxed);
                return Err("Failed to connect".into());
            }
        };

        let (mut write, mut read) = ws_stream.split();
        
        // Start message receiver
        let metrics = self.metrics.clone();
        let receive_task = tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(_)) => {
                        metrics.messages_received.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(_) => {
                        metrics.errors.fetch_add(1, Ordering::Relaxed);
                        break;
                    }
                    _ => {}
                }
            }
        });

        // Simulate user behavior
        let mut action_count = 0;
        while !self.should_stop.load(Ordering::Relaxed) && action_count < self.config.actions_per_game {
            let start = Instant::now();
            
            // Send action
            let action = json!({
                "action": "test_action",
                "data": {
                    "user": self.id,
                    "count": action_count
                }
            });
            
            if write.send(Message::Text(action.to_string())).await.is_ok() {
                self.metrics.messages_sent.fetch_add(1, Ordering::Relaxed);
            } else {
                self.metrics.errors.fetch_add(1, Ordering::Relaxed);
                break;
            }
            
            // Record latency
            self.metrics.record_latency(start.elapsed());
            
            action_count += 1;
            sleep(Duration::from_millis(self.config.message_interval_ms)).await;
        }

        // Clean shutdown
        let _ = write.close().await;
        receive_task.abort();
        
        if action_count >= self.config.actions_per_game {
            self.metrics.games_completed.fetch_add(1, Ordering::Relaxed);
        }
        
        Ok(())
    }
}

/// High-load test runner
pub struct LoadTestRunner {
    config: LoadTestConfig,
    metrics: Arc<LoadTestMetrics>,
    should_stop: Arc<AtomicBool>,
}

impl LoadTestRunner {
    pub fn new(config: LoadTestConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(LoadTestMetrics::default()),
            should_stop: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Starting high-load test with {} concurrent users", self.config.concurrent_users);
        
        // Create lobbies first
        self.create_test_lobbies().await?;
        
        // Rate limiter to control connection rate
        let semaphore = Arc::new(Semaphore::new(10)); // Max 10 connections per batch
        let mut tasks = JoinSet::new();
        
        // Start timer
        let start = Instant::now();
        let test_duration = self.config.test_duration;
        let should_stop = self.should_stop.clone();
        
        // Schedule stop
        tokio::spawn(async move {
            sleep(test_duration).await;
            should_stop.store(true, Ordering::Relaxed);
        });
        
        // Create users
        for i in 0..self.config.concurrent_users {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let user = SimulatedUser {
                id: format!("user-{}-{}", i % self.config.lobby_count, i),
                config: self.config.clone(),
                metrics: self.metrics.clone(),
                should_stop: self.should_stop.clone(),
            };
            
            tasks.spawn(async move {
                let result = user.run().await;
                drop(permit); // Release permit
                result
            });
            
            // Stagger connections
            if i % 10 == 0 {
                sleep(Duration::from_millis(100)).await;
            }
        }
        
        // Wait for all users to complete
        let mut completed = 0;
        let mut failed = 0;
        while let Some(result) = tasks.join_next().await {
            match result {
                Ok(Ok(())) => completed += 1,
                _ => failed += 1,
            }
        }
        
        let elapsed = start.elapsed();
        println!("\nTest completed in {:.2} seconds", elapsed.as_secs_f64());
        println!("Users completed: {}, failed: {}", completed, failed);
        
        // Print metrics
        self.metrics.print_summary();
        
        // Calculate throughput
        let messages_per_sec = self.metrics.messages_sent.load(Ordering::Relaxed) as f64 / elapsed.as_secs_f64();
        println!("Message throughput: {:.2} messages/sec", messages_per_sec);
        
        Ok(())
    }

    async fn create_test_lobbies(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Creating {} test lobbies...", self.config.lobby_count);
        
        for i in 0..self.config.lobby_count {
            let response = reqwest::Client::new()
                .post(format!("{}/api/lobbies", self.config.server_url.replace("ws", "http")))
                .json(&json!({
                    "name": format!("Test Lobby {}", i),
                    "id": format!("test-{}", i), // Force specific ID for testing
                }))
                .send()
                .await?;
                
            if !response.status().is_success() {
                eprintln!("Failed to create lobby {}: {}", i, response.status());
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Run manually with: cargo test high_load_basic -- --ignored --nocapture
    async fn high_load_basic() {
        let config = LoadTestConfig {
            concurrent_users: 50,
            lobby_count: 5,
            test_duration: Duration::from_secs(30),
            ..Default::default()
        };
        
        let runner = LoadTestRunner::new(config);
        runner.run().await.expect("Load test failed");
    }

    #[tokio::test]
    #[ignore] // Run manually with: cargo test high_load_stress -- --ignored --nocapture
    async fn high_load_stress() {
        let config = LoadTestConfig {
            concurrent_users: 200,
            lobby_count: 20,
            message_interval_ms: 50,
            test_duration: Duration::from_secs(60),
            ..Default::default()
        };
        
        let runner = LoadTestRunner::new(config);
        runner.run().await.expect("Load test failed");
    }

    #[tokio::test]
    #[ignore] // Run manually with: cargo test high_load_burst -- --ignored --nocapture
    async fn high_load_burst() {
        // Test burst connections
        let config = LoadTestConfig {
            concurrent_users: 500,
            lobby_count: 10,
            message_interval_ms: 1000, // Slow messages
            test_duration: Duration::from_secs(20),
            ..Default::default()
        };
        
        let runner = LoadTestRunner::new(config);
        runner.run().await.expect("Load test failed");
    }

    #[tokio::test] 
    #[ignore] // Run manually
    async fn connection_limit_test() {
        // Test connection pool limits (3 per user, 100 total)
        let config = LoadTestConfig {
            concurrent_users: 120, // Should hit global limit
            lobby_count: 1,
            test_duration: Duration::from_secs(10),
            ..Default::default()
        };
        
        let runner = LoadTestRunner::new(config);
        runner.run().await.expect("Load test failed");
        
        // Verify that some connections were rejected
        assert!(runner.metrics.connections_failed.load(Ordering::Relaxed) > 0);
    }

    #[tokio::test]
    #[ignore] // Run manually
    async fn memory_leak_test() {
        // Long-running test to check for memory leaks
        let config = LoadTestConfig {
            concurrent_users: 50,
            lobby_count: 5,
            test_duration: Duration::from_secs(300), // 5 minutes
            actions_per_game: 1000,
            ..Default::default()
        };
        
        println!("Starting 5-minute memory leak test...");
        println!("Monitor server memory usage with: watch -n 1 'ps aux | grep bluefelt'");
        
        let runner = LoadTestRunner::new(config);
        runner.run().await.expect("Load test failed");
    }
}