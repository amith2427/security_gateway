use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::Json,
    routing::get,
    Router,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

// ==========================================
// 1. ROLES & AUTHENTICATION (RBAC)
// ==========================================
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Role {
    Citizen,
    Officer,
    Bank,
    Planner,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: Role,
    pub exp: usize,
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the Authorization header (passed down by Kong)
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|val| val.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "));

        let token = match auth_header {
            Some(t) => t,
            // For local testing without Kong, we bypass token validation if no header is sent
            // IN PRODUCTION: Change this to return the Err((StatusCode::UNAUTHORIZED...))
            None => return Ok(Claims {
                sub: "test-user".to_string(),
                role: Role::Citizen, // Change this to test different roles locally!
                exp: 9999999999,
            }),
        };

        // Decode the JWT
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret("super_secret_key".as_ref()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid Token"))?;

        Ok(token_data.claims)
    }
}

// ==========================================
// 2. FIELD-LEVEL PRIVACY & DATA MASKING
// ==========================================
#[derive(Serialize, Deserialize, Clone)]
pub struct LegalRecord {
    pub spatial_unit_id: String,
    pub synthetic_kyc_id: String,
    pub owner_name: Option<String>,
    pub financial_liens: Option<String>,
}

impl LegalRecord {
    pub fn apply_disclosure_controls(&mut self, user_role: &Role) {
        match user_role {
            Role::Citizen | Role::Planner => {
                self.owner_name = Some("[REDACTED]".to_string());
                self.financial_liens = None;
            }
            Role::Officer | Role::Bank => {
                // Full visibility permitted
            }
        }
    }
}

// ==========================================
// 3. TAMPER-EVIDENT AUDIT LOGGING
// ==========================================
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditRecord {
    pub correlation_id: String,
    pub previous_hash: String,
    pub timestamp: u64,
    pub action_payload: String,
    pub current_hash: String,
}

impl AuditRecord {
    pub fn create_event(correlation_id: String, action: String, prev_hash: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let data_to_hash = format!("{}|{}|{}|{}", correlation_id, prev_hash, timestamp, action);
        
        let mut hasher = Sha256::new();
        hasher.update(data_to_hash.as_bytes());
        let current_hash = hex::encode(hasher.finalize());

        Self {
            correlation_id,
            previous_hash: prev_hash,
            timestamp,
            action_payload: action,
            current_hash,
        }
    }
}

// ==========================================
// 4. MAIN ROUTE LOGIC & SERVER
// ==========================================
async fn fetch_from_legal_ledger(spatial_id: &str) -> LegalRecord {
    LegalRecord {
        spatial_unit_id: spatial_id.to_string(),
        synthetic_kyc_id: "SYNC-KYC-99382".to_string(),
        owner_name: Some("Jane Doe".to_string()),
        financial_liens: Some("$50,000 Mortgage via State Bank".to_string()),
    }
}

async fn get_legal_summary(
    claims: Claims,
) -> Result<Json<LegalRecord>, (StatusCode, String)> {
    
    let target_spatial_id = "3D-ULPIN-12345";

    // RBAC Check
    if claims.role == Role::Planner {
        return Err((
            StatusCode::FORBIDDEN,
            "Planners restricted to aggregate data".to_string(),
        ));
    }

    // Fetch raw data
    let mut record = fetch_from_legal_ledger(target_spatial_id).await;

    // Scrub sensitive fields
    record.apply_disclosure_controls(&claims.role);

    // Generate Audit Log
    let last_db_hash = "abc123previoushash...".to_string(); 
    let audit_event = AuditRecord::create_event(
        "local-test-req-id".to_string(),
        format!("READ access by role: {:?}", claims.role),
        last_db_hash,
    );
    
    println!("--> Audit Event Logged: {:#?}", audit_event);

    Ok(Json(record))
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/api/v1/legal/summary", get(get_legal_summary));

    println!("Group 5 Security Gateway running on http://127.0.0.1:8082");
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8082").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}