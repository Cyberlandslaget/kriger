// SPDX-License-Identifier: AGPL-3.0-only
// Copyright Authors of kriger

use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;

use crate::models;

pub struct KrigerClient {
    client: reqwest::Client,
    url: String,
}

impl KrigerClient {
    pub fn new(url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            url,
        }
    }

    pub async fn get_server_config(
        &self,
    ) -> reqwest::Result<models::responses::AppResponse<models::AppConfig>> {
        let url = format!("{}/config/server", &self.url);
        self.request(reqwest::Method::GET, url).await
    }

    pub async fn get_competition_teams(
        &self,
    ) -> reqwest::Result<models::responses::AppResponse<HashMap<String, models::Team>>> {
        let url = format!("{}/competition/teams", &self.url);
        self.request(reqwest::Method::GET, url).await
    }

    pub async fn get_competition_services(
        &self,
    ) -> reqwest::Result<models::responses::AppResponse<Vec<models::Service>>> {
        let url = format!("{}/competition/services", &self.url);
        self.request(reqwest::Method::GET, url).await
    }

    pub async fn get_competition_flag_hints(
        &self,
        service_name: String,
    ) -> reqwest::Result<models::responses::AppResponse<Vec<models::FlagHint>>> {
        let url = format!("{}/competition/flag_hints", &self.url);
        self.request_with_query(
            reqwest::Method::GET,
            url,
            &models::requests::FlagHintQuery {
                service: service_name,
            },
        )
        .await
    }

    pub async fn update_exploit(
        &self,
        exploit: &models::Exploit,
    ) -> reqwest::Result<models::responses::AppResponse<()>> {
        let url = format!("{}/exploits/{}", &self.url, &exploit.manifest.name);
        self.request_with_body(reqwest::Method::PUT, url, exploit)
            .await
    }

    pub async fn execute_exploit(
        &self,
        exploit_name: &str,
    ) -> reqwest::Result<models::responses::AppResponse<()>> {
        let url = format!("{}/exploits/{}/execute", &self.url, &exploit_name);
        self.request(reqwest::Method::POST, url).await
    }

    pub async fn submit_flags(
        &self,
        flags: Vec<String>,
    ) -> reqwest::Result<models::responses::AppResponse<()>> {
        let url = format!("{}/flags", &self.url);
        self.request_with_body(
            reqwest::Method::POST,
            url,
            &models::requests::FlagSubmitRequest { flags },
        )
        .await
    }

    async fn request<U, R>(
        &self,
        method: reqwest::Method,
        url: U,
    ) -> reqwest::Result<models::responses::AppResponse<R>>
    where
        U: reqwest::IntoUrl,
        R: DeserializeOwned + Serialize + ?Sized,
    {
        let response = self.client.request(method, url).send().await?;
        let response: models::responses::AppResponse<R> = response.json().await?;
        Ok(response)
    }

    async fn request_with_query<U, R, Q>(
        &self,
        method: reqwest::Method,
        url: U,
        query: &Q,
    ) -> reqwest::Result<models::responses::AppResponse<R>>
    where
        U: reqwest::IntoUrl,
        R: DeserializeOwned + Serialize + ?Sized,
        Q: Serialize,
    {
        let response = self.client.request(method, url).query(query).send().await?;
        let response: models::responses::AppResponse<R> = response.json().await?;
        Ok(response)
    }

    async fn request_with_body<U, B, R>(
        &self,
        method: reqwest::Method,
        url: U,
        body: &B,
    ) -> reqwest::Result<models::responses::AppResponse<R>>
    where
        U: reqwest::IntoUrl,
        B: Serialize + ?Sized,
        R: DeserializeOwned + Serialize + ?Sized,
    {
        let response = self.client.request(method, url).json(body).send().await?;
        let response: models::responses::AppResponse<R> = response.json().await?;
        Ok(response)
    }
}
