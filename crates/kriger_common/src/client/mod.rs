use serde::{de::DeserializeOwned, Serialize};

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

    pub async fn update_exploit(
        &self,
        exploit: &models::Exploit,
    ) -> reqwest::Result<models::responses::AppResponse<()>> {
        let url = format!("{}/exploits/{}", self.url, &exploit.manifest.name);
        self.request_with_body(reqwest::Method::PUT, url, exploit)
            .await
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
