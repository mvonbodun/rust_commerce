// MongoDB Initialization Script for Rust Commerce
// This script creates the necessary databases and users for each service

// Switch to admin database
db = db.getSiblingDB('admin');

// Create databases for each service
const databases = ['catalog', 'inventory', 'orders', 'price'];

databases.forEach(function(dbName) {
    print(`Creating database: ${dbName}`);
    
    // Switch to the database (creates it if it doesn't exist)
    let serviceDb = db.getSiblingDB(dbName);
    
    // Create a dummy collection to ensure the database is created
    serviceDb.createCollection('_init');
    
    // Remove the dummy collection
    serviceDb._init.drop();
    
    print(`Database ${dbName} created successfully`);
});

// Create a general user for the rust commerce application
db.createUser({
    user: "rust_commerce_user",
    pwd: "rust_commerce_pass",
    roles: [
        { role: "readWrite", db: "catalog" },
        { role: "readWrite", db: "inventory" },
        { role: "readWrite", db: "orders" },
        { role: "readWrite", db: "price" }
    ]
});

print("MongoDB initialization completed successfully!");
print("Created databases: catalog, inventory, orders, price");
print("Created user: rust_commerce_user with readWrite access to all service databases");
