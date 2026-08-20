//! Resource Matching Engine
//! 
//! Matches resource requests with available offers based on
//! capacity, price, trust score, and availability.

use crate::{types::PeerInfo, Match, ResourceOffer, ResourceRequest};
use anyhow::Result;

/// Engine for matching requests with resource offers
pub struct Matcher;

impl Matcher {
    pub fn new() -> Self {
        Self
    }
    
    /// Find best matches for a resource request
    pub async fn find_matches(
        &self,
        request: ResourceRequest,
        peers: Vec<PeerInfo>,
    ) -> Result<Vec<Match>> {
        let mut matches = Vec::new();
        
        for peer in peers {
            for offer in peer.resources {
                if let Some(score) = Self::calculate_match_score(&request, &offer) {
                    let estimated_cost = score * offer.price_per_hour * request.needed_duration_hours as f64;
                    
                    matches.push(Match {
                        offer,
                        request: request.clone(),
                        score,
                        estimated_cost,
                    });
                }
            }
        }
        
        // Sort by score descending
        matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        Ok(matches)
    }
    
    /// Calculate match score between request and offer
    fn calculate_match_score(request: &ResourceRequest, offer: &ResourceOffer) -> Option<f64> {
        // Type must match
        if request.resource_type != offer.resource_type {
            return None;
        }
        
        // Price must be acceptable
        if offer.price_per_hour > request.max_price_per_hour {
            return None;
        }
        
        // Capacity must be sufficient
        if offer.capacity.amount < request.min_capacity.amount {
            return None;
        }
        
        // Calculate quality score (0.0 - 1.0)
        let price_score = 1.0 - (offer.price_per_hour / request.max_price_per_hour);
        let capacity_score = (offer.capacity.amount as f64 / request.min_capacity.amount as f64)
            .min(2.0) / 2.0; // Cap at 2x capacity
        
        // Combined score (can be weighted)
        let score = (price_score * 0.5) + (capacity_score * 0.5);
        
        Some(score.min(1.0).max(0.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PeerId, ResourceCapacity, ResourceType};
    
    fn create_test_offer(price: f64, amount: u32) -> ResourceOffer {
        ResourceOffer {
            peer_id: PeerId::generate(),
            resource_type: ResourceType::Ram,
            capacity: ResourceCapacity {
                amount,
                unit: "GB".to_string(),
                gpu_model: None,
            },
            price_per_hour: price,
            availability: crate::types::AvailabilityWindow {
                start_time: chrono::Utc::now(),
                duration_hours: 24,
            },
        }
    }
    
    fn create_test_request(max_price: f64, min_amount: u32) -> ResourceRequest {
        ResourceRequest {
            requester_id: PeerId::generate(),
            resource_type: ResourceType::Ram,
            min_capacity: ResourceCapacity {
                amount: min_amount,
                unit: "GB".to_string(),
                gpu_model: None,
            },
            max_price_per_hour: max_price,
            needed_duration_hours: 2,
            deadline: chrono::Utc::now() + chrono::Duration::hours(24),
        }
    }
    
    #[test]
    fn test_match_score() {
        let request = create_test_request(1.0, 16);
        let offer = create_test_offer(0.5, 32);
        
        let score = Matcher::calculate_match_score(&request, &offer);
        assert!(score.is_some());
        assert!(score.unwrap() > 0.0);
    }
    
    #[test]
    fn test_no_match_expensive() {
        let request = create_test_request(1.0, 16);
        let offer = create_test_offer(2.0, 32); // Too expensive
        
        let score = Matcher::calculate_match_score(&request, &offer);
        assert!(score.is_none());
    }
}
