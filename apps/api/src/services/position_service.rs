use std::sync::Arc;

use domain_types::PositionSnapshot;

use crate::dto::PositionFilterQuery;
use crate::error::ApiError;
use crate::persistence::SnapshotRepository;
use crate::providers::PositionProvider;

#[derive(Clone)]
pub struct PositionService {
    provider: Arc<dyn PositionProvider>,
    repository: SnapshotRepository,
}

impl PositionService {
    pub fn new(provider: Arc<dyn PositionProvider>, repository: SnapshotRepository) -> Self {
        Self {
            provider,
            repository,
        }
    }

    pub async fn get_position(&self, id: &str) -> Result<PositionSnapshot, ApiError> {
        // Try repository cache first
        if let Some(pos) = self.repository.get_snapshot(id).await? {
            return Ok(pos);
        }

        // Fetch from provider
        let pos = self
            .provider
            .get_position(id)
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("Position '{}' not found", id)))?;

        // Cache snapshot
        let _ = self.repository.save_snapshot(&pos).await;

        Ok(pos)
    }

    pub async fn list_positions(
        &self,
        query: &PositionFilterQuery,
    ) -> Result<Vec<PositionSnapshot>, ApiError> {
        self.provider
            .list_positions(
                query.owner.as_deref(),
                query.protocol.as_deref(),
                query.network.as_deref(),
            )
            .await
    }
}
