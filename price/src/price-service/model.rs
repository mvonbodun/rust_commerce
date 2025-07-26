use bson::Decimal128;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Offer {
    #[serde(rename = "_id")]
    pub id: Option<String>,
    pub sku: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub start_date: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub end_date: DateTime<Utc>,
    pub min_quantity: i32,
    pub max_quantity: Option<i32>,
    pub offer_prices: Vec<OfferPrice>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OfferPrice {
    pub price: Decimal128,
    pub currency: iso_currency::Currency,
}

impl Offer {
    pub fn builder() -> OfferBuilder {
        OfferBuilder::default()
    }
}

#[derive(Default)]
pub struct OfferBuilder {
    id: Option<String>,
    sku: String,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    min_quantity: i32,
    max_quantity: Option<i32>,
    offer_prices: Vec<OfferPrice>,
}

impl OfferBuilder {
    pub fn new(
        sku: String,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        min_quantity: i32,
        offer_prices: Vec<OfferPrice>,
    ) -> Self {
        OfferBuilder {
            id: Some(Uuid::new_v4().to_string()),
            sku,
            start_date,
            end_date,
            min_quantity,
            max_quantity: None,
            offer_prices,
        }
    }
    pub fn sku(&mut self, sku: String) -> &mut Self {
        self.sku = sku;
        self
    }
    pub fn max_quantity(&mut self, max_quantity: i32) -> &mut Self {
        self.max_quantity = Some(max_quantity);
        self
    }
    pub fn build(&mut self) -> Offer {
        Offer {
            id: self.id.clone(),
            sku: self.sku.clone(),
            start_date: self.start_date,
            end_date: self.end_date,
            min_quantity: self.min_quantity,
            max_quantity: self.max_quantity,
            offer_prices: self.offer_prices.clone(),
        }
    }
}

#[derive(Debug, Error)]
pub enum DBError {
    #[error("Database connection error")]
    Connection,
    #[error("Database query error")]
    Query,
    #[error("Database transaction error")]
    Transaction,
    #[error("Database error occurred")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;
    use std::str::FromStr;

    use super::*;

    #[test]
    fn offer_builder_test() {
        let offer = OfferBuilder::new(
            "SKU123".to_string(),
            Utc::now(),
            Utc::now().with_month(12).unwrap(),
            1,
            vec![
                OfferPrice {
                    price: Decimal128::from_str("1.00").unwrap(),
                    currency: iso_currency::Currency::USD,
                },
                OfferPrice {
                    price: Decimal128::from_str("2.00").unwrap(),
                    currency: iso_currency::Currency::EUR,
                },
            ],
        )
        .build();
        println!("Offer: {:?}", offer);
    }
}
