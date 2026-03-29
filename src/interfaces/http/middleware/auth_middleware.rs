use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::Method,
    Error, HttpMessage,
};
use actix_web::body::EitherBody;
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};

use crate::infrastructure::auth::jwt_service::JwtService;

const PUBLIC_PREFIXES: &[&str] = &[
    "/api/users/signup",
    "/api/users/login",
    "/api/products",
    "/api/assets",
    "/api/flights",
    "/api/calculator",
    "/api/health",
];

pub struct AuthMiddleware {
    pub jwt_secret: String,
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service,
            jwt_secret: self.jwt_secret.clone(),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
    jwt_secret: String,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if req.method() == Method::OPTIONS {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res.map_into_left_body())
            });
        }

        if PUBLIC_PREFIXES.iter().any(|p| req.path().starts_with(p)) {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res.map_into_left_body())
            });
        }

        let token = req
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .map(str::to_string);

        let Some(token) = token else {
            return Box::pin(async move {
                let response = actix_web::HttpResponse::Unauthorized()
                    .json(serde_json::json!({ "error": "Missing authorization token" }));
                Ok(req.into_response(response).map_into_right_body())
            });
        };

        let jwt = JwtService::new(self.jwt_secret.clone());
        match jwt.validate_token(&token) {
            Ok(claims) => {
                req.extensions_mut().insert(claims);
                let fut = self.service.call(req);
                Box::pin(async move {
                    let res = fut.await?;
                    Ok(res.map_into_left_body())
                })
            }
            Err(_) => Box::pin(async move {
                let response = actix_web::HttpResponse::Unauthorized()
                    .json(serde_json::json!({ "error": "Invalid or expired token" }));
                Ok(req.into_response(response).map_into_right_body())
            }),
        }
    }
}
