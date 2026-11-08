pub mod health;
pub mod markets;
pub mod openapi;
pub mod oracles;
pub mod positions;
pub mod recovery;

pub use health::{health_handler, ready_handler};
pub use markets::list_markets_handler;
pub use openapi::{openapi_json_handler, ApiDoc};
pub use oracles::get_oracle_price_handler;
pub use positions::{get_position_handler, get_position_risk_handler, list_positions_handler};
pub use recovery::{
    get_recovery_recommendation_handler, optimize_recovery_handler, quote_recovery_handler,
};
