use chrono::NaiveDate;
use log::{debug, error};
use std::collections::HashMap;

use crate::model::Offer;
use crate::persistence::offer_dao::OfferDao;

pub enum HandlerError {
    InternalError(String),
    ValidationError(String),
}

pub async fn create_offer(
    offer: Offer,
    offer_dao: &(dyn OfferDao + Sync + Send),
) -> Result<Offer, HandlerError> {
    debug!("Before call to create_offer");

    let result = offer_dao.create_offer(offer).await;
    match result {
        Ok(offer) => Ok(offer),
        Err(e) => {
            error!("Error creating offer: {e}");
            Err(HandlerError::InternalError(format!("Failed to create offer: {e}")))
        }
    }
}

pub async fn get_offer(
    offer_id: String,
    offer_dao: &(dyn OfferDao + Sync + Send),
) -> Result<Option<Offer>, HandlerError> {
    debug!("Before call to get offer");
    let result = offer_dao.get_offer(offer_id).await;
    debug!("After call to get offer: {result:?}");

    match result {
        Ok(Some(offer)) => Ok(Some(offer)),
        Ok(None) => Ok(None),
        Err(e) => {
            error!("Error getting offer: {e}");
            Err(HandlerError::InternalError(format!("Failed to get offer: {e}")))
        }
    }
}

pub async fn delete_offer(
    offer_id: String,
    offer_dao: &(dyn OfferDao + Send + Sync),
) -> Result<(), HandlerError> {
    let result = offer_dao.delete_offer(offer_id).await;
    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            error!("Error deleting offer: {e}");
            Err(HandlerError::InternalError(format!("Failed to delete offer: {e}")))
        }
    }
}

pub async fn get_best_offer_price(
    sku: String,
    quantity: i32,
    date: Option<String>,
    currency: String,
    offer_dao: &(dyn OfferDao + Send + Sync),
) -> Result<Option<Offer>, HandlerError> {
    debug!("Before call to get_best_offer_price");

    // Validate input parameters
    if sku.trim().is_empty() {
        return Err(HandlerError::ValidationError(
            "SKU cannot be empty".to_string(),
        ));
    }

    if quantity <= 0 {
        return Err(HandlerError::ValidationError(
            "Quantity must be positive".to_string(),
        ));
    }

    // Validate currency (only USD and EUR allowed)
    if currency != "USD" && currency != "EUR" {
        return Err(HandlerError::ValidationError(
            "Currency must be USD or EUR".to_string(),
        ));
    }

    // Parse date or use current date
    let parsed_date = match date {
        Some(date_str) => match NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
            Ok(date) => date,
            Err(_) => {
                return Err(HandlerError::ValidationError(
                    "Date must be in YYYY-MM-DD format".to_string(),
                ));
            }
        },
        None => chrono::Utc::now().date_naive(),
    };

    debug!("Validated parameters - sku: {sku}, quantity: {quantity}, date: {parsed_date}, currency: {currency}");

    // Call DAO method
    let result = offer_dao
        .find_best_offer_price(&sku, quantity, parsed_date, &currency)
        .await;

    match result {
        Ok(offer) => {
            debug!("Successfully found best offer: {offer:?}");
            Ok(offer)
        }
        Err(e) => {
            error!("Error finding best offer price: {e}");
            Err(HandlerError::InternalError(format!(
                "Failed to find best offer price: {e}"
            )))
        }
    }
}

pub async fn get_best_offer_prices(
    skus: Vec<String>,
    quantity: i32,
    date: Option<String>,
    currency: String,
    offer_dao: &(dyn OfferDao + Send + Sync),
) -> Result<HashMap<String, Option<Offer>>, HandlerError> {
    debug!(
        "Before call to get_best_offer_prices for {} SKUs",
        skus.len()
    );

    // Validate input parameters
    if skus.is_empty() {
        return Err(HandlerError::ValidationError(
            "SKUs list cannot be empty".to_string(),
        ));
    }

    if skus.len() > 100 {
        return Err(HandlerError::ValidationError(format!(
            "Too many SKUs provided. Maximum is 100, got {}",
            skus.len()
        )));
    }

    // Validate that all SKUs are non-empty
    for sku in &skus {
        if sku.trim().is_empty() {
            return Err(HandlerError::ValidationError(
                "All SKUs must be non-empty".to_string(),
            ));
        }
    }

    if quantity <= 0 {
        return Err(HandlerError::ValidationError(
            "Quantity must be positive".to_string(),
        ));
    }

    // Validate currency (only USD and EUR allowed)
    if currency != "USD" && currency != "EUR" {
        return Err(HandlerError::ValidationError(
            "Currency must be USD or EUR".to_string(),
        ));
    }

    // Parse date or use current date
    let parsed_date = match date {
        Some(date_str) => match NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
            Ok(date) => date,
            Err(_) => {
                return Err(HandlerError::ValidationError(
                    "Date must be in YYYY-MM-DD format".to_string(),
                ));
            }
        },
        None => chrono::Utc::now().date_naive(),
    };

    debug!("Validated parameters - {} SKUs, quantity: {quantity}, date: {parsed_date}, currency: {currency}", skus.len());

    // Call DAO method
    let result = offer_dao
        .find_best_offer_prices(&skus, quantity, parsed_date, &currency)
        .await;

    match result {
        Ok(offers) => {
            debug!("Successfully found best offers for {} SKUs", offers.len());
            Ok(offers)
        }
        Err(e) => {
            error!("Error finding best offer prices: {e}");
            Err(HandlerError::InternalError(format!(
                "Failed to find best offer prices: {e}"
            )))
        }
    }
}
