use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::{ready, LocalBoxFuture, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::task::{Context, Poll};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
}

pub struct Authentication;

impl<S, B> Transform<S, ServiceRequest> for Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthenticationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationMiddleware { service }))
    }
}

pub struct AuthenticationMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Skip authentication for certain paths
        if should_skip_auth(&req) {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            });
        }

        // Get the token from the Authorization header
        let auth_header = match req.headers().get("Authorization") {
            Some(header) => header,
            None => {
                return Box::pin(async move {
                    Err(actix_web::error::ErrorUnauthorized("Missing authorization header"))
                })
            }
        };

        let auth_str = match auth_header.to_str() {
            Ok(str) => str,
            Err(_) => {
                return Box::pin(async move {
                    Err(actix_web::error::ErrorUnauthorized("Invalid authorization header"))
                })
            }
        };

        if !auth_str.starts_with("Bearer ") {
            return Box::pin(async move {
                Err(actix_web::error::ErrorUnauthorized("Invalid authorization scheme"))
            });
        }

        let token = &auth_str[7..];

        // Validate JWT token
        match validate_token(token) {
            Ok(claims) => {
                // Add claims to request extensions
                req.extensions_mut().insert(claims);
                let fut = self.service.call(req);
                Box::pin(async move {
                    let res = fut.await?;
                    Ok(res)
                })
            }
            Err(_) => Box::pin(async move {
                Err(actix_web::error::ErrorUnauthorized("Invalid token"))
            }),
        }
    }
}

fn should_skip_auth(req: &ServiceRequest) -> bool {
    let path = req.path();
    // List of paths that don't require authentication
    let public_paths = vec![
        "/api/v1/health",
        "/api/v1/auth/login",
        "/api/v1/auth/register",
    ];
    public_paths.iter().any(|&p| path.starts_with(p))
}

fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-256-bit-secret".to_string());
    let key = DecodingKey::from_secret(secret.as_bytes());
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    let token_data = decode::<Claims>(token, &key, &validation)?;
    Ok(token_data.claims)
} 