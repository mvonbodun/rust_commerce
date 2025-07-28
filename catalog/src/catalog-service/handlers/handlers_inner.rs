use log::{debug, error};
use crate::model::{Product, ProductBuilder, Reviews, HierarchicalCategories, ProductVariant, Packaging};
use crate::persistence::product_dao::ProductDao;
use crate::catalog_messages::{ProductCreateRequest, ProductUpdateRequest};

pub enum HandlerError {
    InternalError(String),
}

impl HandlerError {
    pub fn default_internal_error() -> Self {
        HandlerError::InternalError("An unexpected error occurred. Please try again".to_owned())
    }
}

pub async fn create_product(
    request: ProductCreateRequest,
    created_by: Option<String>,
    product_dao: &(dyn ProductDao + Sync + Send),
) -> Result<Product, HandlerError> {
    debug!("Before call to create_product handler_inner");
    
    let mut product_builder = ProductBuilder::new(request.name, request.product_ref);
    
    if let Some(brand) = request.brand {
        product_builder.brand(brand);
    }
    
    if let Some(slug) = request.slug {
        product_builder.slug(slug);
    }
    
    if let Some(long_description) = request.long_description {
        product_builder.long_description(long_description);
    }
    
    if let Some(product_type) = request.product_type {
        product_builder.product_type(product_type);
    }
    
    if let Some(seo_title) = request.seo_title {
        product_builder.seo_title(seo_title);
    }
    
    if let Some(seo_description) = request.seo_description {
        product_builder.seo_description(seo_description);
    }
    
    if let Some(seo_keywords) = request.seo_keywords {
        product_builder.seo_keywords(seo_keywords);
    }
    
    product_builder.display_on_site(request.display_on_site);
    
    if let Some(tax_code) = request.tax_code {
        product_builder.tax_code(tax_code);
    }
    
    if let Some(created_by) = created_by {
        product_builder.created_by(created_by);
    }

    // Map related products
    product_builder.related_products(request.related_products);
    
    // Map reviews if present
    if let Some(proto_reviews) = request.reviews {
        let reviews = Reviews {
            bayesian_avg: proto_reviews.bayesian_avg.into(),
            count: proto_reviews.count,
            rating: proto_reviews.rating,
        };
        debug!("Mapped reviews: {:?}", reviews);
        product_builder.reviews(reviews);
    }
    
    // Map hierarchical categories if present
    if let Some(proto_hc) = request.hierarchical_categories {
        let hc = HierarchicalCategories {
            lvl0: proto_hc.lvl0,
            lvl1: proto_hc.lvl1,
            lvl2: proto_hc.lvl2,
        };
        product_builder.hierarchical_categories(hc);
    }
    
    // Map list categories
    product_builder.list_categories(request.list_categories);
    
    // Map defining attributes
    product_builder.defining_attributes(request.defining_attributes);
    
    // Map descriptive attributes
    product_builder.descriptive_attributes(request.descriptive_attributes);
    
    // Map default variant
    if let Some(default_variant) = request.default_variant {
        product_builder.default_variant(default_variant);
    }
    
    // Map variants
    let variants: Vec<ProductVariant> = request.variants.into_iter().map(|proto_variant| {
        ProductVariant {
            sku: proto_variant.sku,
            defining_attributes: Some(proto_variant.defining_attributes),
            abbreviated_color: proto_variant.abbreviated_color,
            abbreviated_size: proto_variant.abbreviated_size,
            height: proto_variant.height,
            width: proto_variant.width,
            length: proto_variant.length,
            weight: proto_variant.weight,
            weight_unit: proto_variant.weight_unit,
            packaging: proto_variant.packaging.map(|proto_packaging| Packaging {
                height: proto_packaging.height,
                width: proto_packaging.width,
                length: proto_packaging.length,
                weight: proto_packaging.weight,
                weight_unit: proto_packaging.weight_unit,
            }),
            image_urls: proto_variant.image_urls,
        }
    }).collect();
    product_builder.variants(variants);

    let product = product_builder.build();
    let result = product_dao.create_product(product).await;

    match result {
        Ok(product) => Ok(product),
        Err(e) => {
            error!("Error creating product: {}", e);
            Err(HandlerError::InternalError(format!(
                "Failed to create product: {}",
                e
            )))
        }
    }
}

pub async fn get_product(
    product_id: String,
    product_dao: &(dyn ProductDao + Sync + Send),
) -> Result<Option<Product>, HandlerError> {
    debug!("Before call to get_product handler_inner");
    let result = product_dao.get_product(&product_id).await;
    debug!("After call to get_product handler_inner: {:?}", result);

    match result {
        Ok(Some(product)) => Ok(Some(product)),
        Ok(None) => Ok(None),
        Err(e) => {
            error!("Error getting product: {}", e);
            Err(HandlerError::InternalError(format!(
                "Failed to get product: {}",
                e
            )))
        }
    }
}

pub async fn update_product(
    product_id: String,
    request: ProductUpdateRequest,
    product_dao: &(dyn ProductDao + Send + Sync),
) -> Result<Option<Product>, HandlerError> {
    debug!("Before call to update_product handler_inner");
    
    // Convert the proto Product to our domain Product
    let product = request.product.ok_or_else(|| {
        HandlerError::InternalError("Product is required in update request".to_string())
    })?;
    
    // Map the proto product to domain product
    let domain_product = Product {
        id: product.id,
        name: product.name,
        long_description: product.long_description,
        brand: product.brand,
        slug: product.slug,
        product_ref: product.product_ref,
        product_type: product.product_type,
        seo_title: product.seo_title,
        seo_description: product.seo_description,
        seo_keywords: product.seo_keywords,
        display_on_site: product.display_on_site,
        tax_code: product.tax_code,
        related_products: product.related_products,
        reviews: product.reviews.map(|proto_reviews| Reviews {
            bayesian_avg: proto_reviews.bayesian_avg.into(),
            count: proto_reviews.count,
            rating: proto_reviews.rating,
        }),
        hierarchical_categories: product.hierarchical_categories.map(|proto_hc| HierarchicalCategories {
            lvl0: proto_hc.lvl0,
            lvl1: proto_hc.lvl1,
            lvl2: proto_hc.lvl2,
        }),
        list_categories: product.list_categories,
        created_at: product.created_at.map(|ts| {
            use chrono::{DateTime, Utc};
            DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos as u32)
                .unwrap_or_else(|| Utc::now())
        }),
        updated_at: product.updated_at.map(|ts| {
            use chrono::{DateTime, Utc};
            DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos as u32)
                .unwrap_or_else(|| Utc::now())
        }),
        created_by: product.created_by,
        updated_by: product.updated_by,
        defining_attributes: product.defining_attributes,
        descriptive_attributes: product.descriptive_attributes,
        default_variant: product.default_variant,
        variants: product.variants.into_iter().map(|proto_variant| ProductVariant {
            sku: proto_variant.sku,
            defining_attributes: Some(proto_variant.defining_attributes),
            abbreviated_color: proto_variant.abbreviated_color,
            abbreviated_size: proto_variant.abbreviated_size,
            height: proto_variant.height,
            width: proto_variant.width,
            length: proto_variant.length,
            weight: proto_variant.weight,
            weight_unit: proto_variant.weight_unit,
            packaging: proto_variant.packaging.map(|proto_packaging| Packaging {
                height: proto_packaging.height,
                width: proto_packaging.width,
                length: proto_packaging.length,
                weight: proto_packaging.weight,
                weight_unit: proto_packaging.weight_unit,
            }),
            image_urls: proto_variant.image_urls,
        }).collect(),
    };
    
    let result = product_dao.update_product(&product_id, domain_product).await;

    match result {
        Ok(Some(product)) => Ok(Some(product)),
        Ok(None) => Ok(None),
        Err(e) => {
            error!("Error updating product: {}", e);
            Err(HandlerError::InternalError(format!(
                "Failed to update product: {}",
                e
            )))
        }
    }
}

pub async fn delete_product(
    product_id: String,
    product_dao: &(dyn ProductDao + Send + Sync),
) -> Result<bool, HandlerError> {
    debug!("Before call to delete_product handler_inner");
    let result = product_dao.delete_product(&product_id).await;

    match result {
        Ok(deleted) => Ok(deleted),
        Err(e) => {
            error!("Error deleting product: {}", e);
            Err(HandlerError::InternalError(format!(
                "Failed to delete product: {}",
                e
            )))
        }
    }
}

pub async fn search_products(
    query: Option<String>,
    categories: Vec<String>,
    brand: Option<String>,
    limit: Option<i64>,
    offset: Option<u64>,
    product_dao: &(dyn ProductDao + Send + Sync),
) -> Result<Vec<Product>, HandlerError> {
    debug!("Before call to search_products handler_inner");
    let result = product_dao.search_products(
        query.as_deref(),
        &categories,
        brand.as_deref(),
        limit,
        offset,
    ).await;

    match result {
        Ok(products) => Ok(products),
        Err(e) => {
            error!("Error searching products: {}", e);
            Err(HandlerError::InternalError(format!(
                "Failed to search products: {}",
                e
            )))
        }
    }
}

pub async fn export_products(
    batch_size: Option<i64>,
    offset: Option<u64>,
    product_dao: &(dyn ProductDao + Send + Sync),
) -> Result<Vec<Product>, HandlerError> {
    debug!("Before call to export_products handler_inner");
    let result = if offset.is_some() {
        product_dao.export_products_batch(batch_size, offset).await
    } else {
        product_dao.export_all_products(batch_size).await
    };

    match result {
        Ok(products) => Ok(products),
        Err(e) => {
            error!("Error exporting products: {}", e);
            Err(HandlerError::InternalError(format!(
                "Failed to export products: {}",
                e
            )))
        }
    }
}
