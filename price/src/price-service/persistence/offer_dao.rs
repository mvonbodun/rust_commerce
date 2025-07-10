use log::{debug, error, info};

use async_trait::async_trait;
use bson::doc;
use mongodb::Collection;

use crate::model::{DBError, Offer};

#[async_trait]
pub trait OfferDao {
    async fn create_offer(&self, offer: Offer) -> Result<Offer, DBError>;
    async fn delete_offer(&self, offer_id: String) -> Result<(), DBError>;
    async fn get_offer(&self, offer_id: String) -> Result<Option<Offer>, DBError>;
}

pub struct OfferDaoImpl {
    collection: Collection<Offer>,
}

impl OfferDaoImpl {
    pub fn new(collection: Collection<Offer>) -> Self {
        OfferDaoImpl { collection }
    }
}

#[async_trait]
impl OfferDao for OfferDaoImpl {
    // Create an offer
    async fn create_offer(&self, offer: Offer) -> Result<Offer, DBError> {
        // Implement logic to create an offer in the database
        // and return the created offer or an error if one occurred
        let insert_result = self.collection.insert_one(&offer).await.map_err(|error| {
            error!("Error on insert: {:?}", error);
            DBError::Other(Box::new(error))
        })?;

        info!("Inserted offer result: {:?}", insert_result);
        debug!("Offer after insert: {:?}", offer);
        Ok(offer)
    }
    // Get an offer
    async fn get_offer(&self, offer_id: String) -> Result<Option<Offer>, DBError> {
        // Implement logic to get an offer from the database
        // by its offer ID and return the offer or an error if one occurred
        debug!("before call to find_one - offer_id: {:?}", offer_id);
        let find_result = self
            .collection
            .find_one(doc! {"_id": &offer_id})
            .await
            .map_err(|error| {
                error!("DB error: {:?}", error);
                DBError::Other(Box::new(error))
            })?;

        match find_result {
            Some(offer) => {
                debug!("Found offer: {:?}", offer);
                Ok(Some(offer))
            }
            None => {
                debug!("Offer not found for offer_id: {:?}", offer_id);
                Ok(None)
            }
        }
    }
    // Delete an offer
    async fn delete_offer(&self, offer_id: String) -> Result<(), DBError> {
        // Implement logic to delete an offer from the database
        // by its offer ID and return a success or an error if one occurred
        let delete_result = self
            .collection
            .delete_one(doc! {"_id": &offer_id})
            .await
            .map_err(|error| {
                error!("Error on delete: {:?}", error);
                DBError::Other(Box::new(error))
            })?;

        info!("Deleted offer result: {:?}", delete_result);
        Ok(())
    }
}
