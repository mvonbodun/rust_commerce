use log::{debug, error};

use crate::model::Offer;
use crate::persistence::offer_dao::OfferDao;

pub enum HandlerError {
    InternalError(String),
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
            error!("Error creating offer: {}", e);
            Err(HandlerError::InternalError(format!(
                "Failed to create offer: {}",
                e
            )))
        }
    }
}

pub async fn get_offer(
    offer_id: String,
    offer_dao: &(dyn OfferDao + Sync + Send),
) -> Result<Option<Offer>, HandlerError> {
    debug!("Before call to get offer");
    let result = offer_dao.get_offer(offer_id).await;
    debug!("After call to get offer: {:?}", result);

    match result {
        Ok(Some(offer)) => Ok(Some(offer)),
        Ok(None) => Ok(None),
        Err(e) => {
            error!("Error getting offer: {}", e);
            Err(HandlerError::InternalError(format!(
                "Failed to get offer: {}",
                e
            )))
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
            error!("Error deleting offer: {}", e);
            Err(HandlerError::InternalError(format!(
                "Failed to delete offer: {}",
                e
            )))
        }
    }
}
