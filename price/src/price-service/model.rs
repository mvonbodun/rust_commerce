use bson::Decimal128;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Offer {
    #[serde(rename = "_id")]
    pub id: Option<String>,
    pub item_id: String,
    pub item_ref: Option<String>,
    pub start_date: DateTime<Utc>,
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
    item_id: String,
    item_ref: Option<String>,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    min_quantity: i32,
    max_quantity: Option<i32>,
    offer_prices: Vec<OfferPrice>,
}

impl OfferBuilder {
    pub fn new(
        item_id: String,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        min_quantity: i32,
        offer_prices: Vec<OfferPrice>,
    ) -> Self {
        OfferBuilder {
            id: Some(Uuid::new_v4().to_string()),
            item_id,
            item_ref: None,
            start_date,
            end_date,
            min_quantity,
            max_quantity: None,
            offer_prices,
        }
    }
    pub fn item_ref(&mut self, item_ref: String) -> &mut Self {
        self.item_ref = Some(item_ref);
        self
    }
    pub fn max_quantity(&mut self, max_quantity: i32) -> &mut Self {
        self.max_quantity = Some(max_quantity);
        self
    }
    pub fn build(&mut self) -> Offer {
        Offer {
            id: self.id.clone(),
            item_id: self.item_id.clone(),
            item_ref: self.item_ref.clone(),
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
            Uuid::new_v4().to_string(),
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
