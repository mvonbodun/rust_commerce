use log::{debug, error, info};

use async_trait::async_trait;
use bson::doc;
use chrono::NaiveDate;
use mongodb::Collection;

use crate::model::{DBError, Offer};

#[async_trait]
pub trait OfferDao {
    async fn create_offer(&self, offer: Offer) -> Result<Offer, DBError>;
    async fn delete_offer(&self, offer_id: String) -> Result<(), DBError>;
    async fn get_offer(&self, offer_id: String) -> Result<Option<Offer>, DBError>;
    async fn find_best_offer_price(
        &self,
        sku: &str,
        quantity: i32,
        date: NaiveDate,
        currency: &str,
    ) -> Result<Option<Offer>, DBError>;
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

    // Find the best offer price for given parameters
    async fn find_best_offer_price(
        &self,
        sku: &str,
        quantity: i32,
        date: NaiveDate,
        currency: &str,
    ) -> Result<Option<Offer>, DBError> {
        debug!(
            "Finding best offer price for sku: {}, quantity: {}, date: {}, currency: {}",
            sku, quantity, date, currency
        );

        // Convert NaiveDate to BSON DateTime for MongoDB query
        let bson_date = bson::DateTime::from_chrono(
            date.and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(chrono::Utc)
                .unwrap(),
        );

        // Build the MongoDB query based on playground-1.mongodb.js line 20
        let query = doc! {
            "sku": sku,
            "min_quantity": { "$lte": quantity },
            "max_quantity": { "$gte": quantity },
            "start_date": { "$lte": bson_date },
            "end_date": { "$gte": bson_date },
            "offer_prices": { "$elemMatch": { "currency": currency } }
        };

        debug!("MongoDB query: {:?}", query);

        // Execute the query with sort and limit using find() instead of find_one() 
        // because find_one() doesn't support sorting
        let find_options = mongodb::options::FindOptions::builder()
            .sort(doc! { "offer_prices.price": 1 })
            .limit(1)
            .build();

        let mut cursor = self
            .collection
            .find(query)
            .with_options(find_options)
            .await
            .map_err(|error| {
                error!("DB error in find_best_offer_price: {:?}", error);
                DBError::Other(Box::new(error))
            })?;

        // Get the first result from the cursor
        use futures::stream::StreamExt;
        let find_result = cursor.next().await;

        match find_result {
            Some(result) => match result {
                Ok(offer) => {
                    debug!("Found best offer: {:?}", offer);
                    Ok(Some(offer))
                }
                Err(error) => {
                    error!("DB cursor error in find_best_offer_price: {:?}", error);
                    Err(DBError::Other(Box::new(error)))
                }
            },
            None => {
                debug!(
                    "No offer found for sku: {}, quantity: {}, date: {}, currency: {}",
                    sku, quantity, date, currency
                );
                Ok(None)
            }
        }
    }
}
