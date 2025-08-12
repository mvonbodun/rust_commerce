//! Tests for the corrected path format using category names instead of slugs
//!
//! This test suite validates that category paths are formatted correctly as:
//! "Electronics > Smartphones > iPhone" instead of "/electronics/smartphones/iphone"

use rust_catalog::model::Category;

#[test]
fn test_root_category_path() {
    let root = Category::new(
        "/electronics".to_string(),
        "Electronics".to_string(),
        "Electronic devices and gadgets".to_string(),
        None,
        1,
    );

    // Root category path should just be the category name
    let path = root.generate_path(&[]);
    assert_eq!(path, "Electronics");
}

#[test]
fn test_two_level_path() {
    let root = Category::new(
        "/electronics".to_string(),
        "Electronics".to_string(),
        "Electronic devices and gadgets".to_string(),
        None,
        1,
    );

    let child = Category::new(
        "/electronics/smartphones".to_string(),
        "Smartphones".to_string(),
        "Mobile phones and accessories".to_string(),
        root.id.clone(),
        1,
    );

    let path = child.generate_path(&[root]);
    assert_eq!(path, "Electronics > Smartphones");
}

#[test]
fn test_three_level_path() {
    let root = Category::new(
        "/men".to_string(),
        "Men".to_string(),
        "Men's clothing and accessories".to_string(),
        None,
        1,
    );

    let level2 = Category::new(
        "/men/mens-apparel".to_string(),
        "Mens Apparel".to_string(),
        "Men's clothing".to_string(),
        root.id.clone(),
        1,
    );

    let level3 = Category::new(
        "/men/mens-apparel/classic-jeans".to_string(),
        "Classic Jeans".to_string(),
        "Classic style jeans".to_string(),
        level2.id.clone(),
        1,
    );

    let path = level3.generate_path(&[root, level2]);
    assert_eq!(path, "Men > Mens Apparel > Classic Jeans");
}

#[test]
fn test_four_level_path() {
    let root = Category::new(
        "/electronics".to_string(),
        "Electronics".to_string(),
        "Electronic devices".to_string(),
        None,
        1,
    );

    let computers = Category::new(
        "/electronics/computers".to_string(),
        "Computers".to_string(),
        "Computing devices".to_string(),
        root.id.clone(),
        1,
    );

    let laptops = Category::new(
        "/electronics/computers/laptops".to_string(),
        "Laptops".to_string(),
        "Portable computers".to_string(),
        computers.id.clone(),
        1,
    );

    let gaming_laptops = Category::new(
        "/electronics/computers/laptops/gaming-laptops".to_string(),
        "Gaming Laptops".to_string(),
        "High-performance gaming laptops".to_string(),
        laptops.id.clone(),
        1,
    );

    let path = gaming_laptops.generate_path(&[root, computers, laptops]);
    assert_eq!(path, "Electronics > Computers > Laptops > Gaming Laptops");
}

#[test]
fn test_path_with_special_characters() {
    let root = Category::new(
        "/home-garden".to_string(),
        "Home & Garden".to_string(),
        "Home and garden products".to_string(),
        None,
        1,
    );

    let kitchen = Category::new(
        "/home-garden/kitchen-dining".to_string(),
        "Kitchen & Dining".to_string(),
        "Kitchen and dining products".to_string(),
        root.id.clone(),
        1,
    );

    let path = kitchen.generate_path(&[root]);
    assert_eq!(path, "Home & Garden > Kitchen & Dining");
}

#[test]
fn test_path_comparison_old_vs_new_format() {
    let root = Category::new(
        "/men".to_string(),
        "Men".to_string(),
        "Men's clothing".to_string(),
        None,
        1,
    );

    let apparel = Category::new(
        "/men/mens-apparel".to_string(),
        "Mens Apparel".to_string(),
        "Men's clothing".to_string(),
        root.id.clone(),
        1,
    );

    let jeans = Category::new(
        "/men/mens-apparel/classic-jeans".to_string(),
        "Classic Jeans".to_string(),
        "Classic style jeans".to_string(),
        apparel.id.clone(),
        1,
    );

    let correct_path = jeans.generate_path(&[root, apparel]);
    
    // This should NOT match the old buggy format
    let old_buggy_format = "/men./men/mens-apparel./men/mens-apparel/classic-jeans";
    assert_ne!(correct_path, old_buggy_format);
    
    // This SHOULD match the new correct format
    let expected_format = "Men > Mens Apparel > Classic Jeans";
    assert_eq!(correct_path, expected_format);
    
    println!("✅ Path format correction verified:");
    println!("   Old buggy format: {}", old_buggy_format);
    println!("   New correct format: {}", correct_path);
}
