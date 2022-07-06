use crate::rest::RestQuery;

use super::config::Configs;
use anyhow::{bail, Result};
use graphql_client::{GraphQLQuery, Response};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};

pub struct AuthorizedClient;

impl AuthorizedClient {
    pub fn new(configs: &Configs) -> Result<Client> {
        let mut headers = HeaderMap::new();
        if let Some(token) = configs.root_config.fly_pat.clone() {
            headers.insert(
                "Authorization",
                HeaderValue::from_str(format!("Bearer {}", token).as_str())?,
            );
        } else {
            bail!("No fly personal access token configured");
        }
        let client = Client::builder().default_headers(headers).build()?;
        Ok(client)
    }
}

pub async fn post_graphql<Q: GraphQLQuery, U: reqwest::IntoUrl>(
    client: &reqwest::Client,
    url: U,
    variables: Q::Variables,
) -> Result<Response<Q::ResponseData>, reqwest::Error> {
    let body = Q::build_query(variables);
    let reqwest_response = client.post(url).json(&body).send().await?;

    Ok(reqwest_response.json().await?)
}

pub async fn post_rest<Q: RestQuery, U: reqwest::IntoUrl>(
    client: &reqwest::Client,
    url: U,
    body: Q::RequestData,
) -> Result<Q::ResponseData, reqwest::Error> {
    let reqwest_response = client.post(url).json(&body).send().await?;

    Ok(reqwest_response.json().await?)
}

pub async fn delete_rest<U: reqwest::IntoUrl>(
    client: &reqwest::Client,
    url: U,
) -> Result<(), reqwest::Error> {
    client.delete(url).send().await?;

    Ok(())
}

pub async fn get_rest<R: for<'de> serde::Deserialize<'de>, U: reqwest::IntoUrl>(
    client: &reqwest::Client,
    url: U,
) -> Result<R, reqwest::Error> {
    let reqwest_response = client.get(url).send().await?;

    Ok(reqwest_response.json().await?)
}
