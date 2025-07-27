<img src="https://r2cdn.perplexity.ai/pplx-full-logo-primary-dark%402x.png" class="logo" width="120"/>

# find a good example of using rustcontainers integration test framework with mongodb with a rust project

A strong example of using the Rust testcontainers integration test framework with MongoDB in a Rust project is detailed in a blog post, where the author outlines how to set up concurrent, self-contained integration tests that work both locally and in CI/CD pipelines. Here’s an overview, with key steps and code snippets adapted from the walkthrough[^1]:

## Project Architecture

- **Database Abstraction:** Use a trait (`DocumentDatabaseConnector`) for database operations.
- **MongoDB Client Struct:** Encapsulates connection details and implements the trait.

```rust
#[async_trait]
pub(crate) trait DocumentDatabaseConnector {
    // trait methods...
}

#[derive(Debug, Clone)]
pub(crate) struct MongoDBClient {
    client: Client,
    db_name: String,
}
```


## Integration Test Setup with Testcontainers and MongoDB

1. **Install Dependencies**
    - In your `Cargo.toml`, add:

```toml
testcontainers = "0.14.0"
mongodb = "2"
```

2. **Test Utilities**
    - **Port Generator:** Ensure each test gets a unique port.

```rust
fn generate_port_number() -> u16 {
   let address = "0.0.0.0:0";
   let socket = UdpSocket::bind(address).expect("Cannot bind to socket");
   let local_addr = socket.local_addr().expect("Cannot get local address");
   local_addr.port()
}
```

    - **MongoDB Container Setup:**

```rust
fn get_mongo_image(port: &u16) -> RunnableImage<GenericImage> {
    let image = GenericImage::new("mongo", "5.0.6");
    RunnableImage::from(image).with_mapped_port((*port, 27017))
}
```

    - **Populate Test Data:** Use a shell script or similar method to seed test data into the test DB.
3. **Example Test**

```rust
#[tokio::test(flavor = "multi_thread")]
async fn find_a_document_by_version_when_one_exists() {
    let docker = clients::Cli::default();
    let port = generate_port_number();
    let mongo_img = get_mongo_image(&port);
    let _c = docker.run(mongo_img);
    
    populate_test_data(&port);
    let uri = format!("mongodb://0.0.0.0:{}", port);
    let db = MongoDBClient::init(uri, "users".to_string()).await;
    
    // ... your test logic here ...
}
```


- Each test can run in parallel, launches its own MongoDB instance (on a random port), imports its test data, and cleans up afterward.


## Benefits

- **Tests are isolated** from each other.
- **No global MongoDB dependency**—tests spin up their own containers.
- Works **locally and on CI** reliably.

For a full example, including the MongoDB connector and test runner code, see the referenced GitHub repo linked in the original post[^1].

This approach leverages the `testcontainers` crate for managing Dockerized dependencies and enables high-confidence, reproducible integration testing with MongoDB in Rust.

[^1]

<div style="text-align: center">⁂</div>

[^1]: https://blog.devgenius.io/concurrent-mongodb-testing-in-rust-2dd606b3d4a1

[^2]: https://dev.to/sergiomarcial/mastering-integration-testing-in-rust-with-testcontainers-3aml

[^3]: https://www.mongodb.com/developer/languages/rust/rust-mongodb-crud-tutorial/

[^4]: https://users.rust-lang.org/t/concurrent-mongodb-testing-in-rust/105164

[^5]: https://github.com/mongodb/mongo-rust-driver

[^6]: https://docs.rs/mongodb/latest/mongodb/

[^7]: https://blog.devgenius.io/concurrent-mongodb-testing-in-rust-2dd606b3d4a1?gi=6308b2d45b85

[^8]: https://friendlyuser.github.io/posts/tech/rust/getting_started_with_mongodb_in_rust/

[^9]: https://www.youtube.com/watch?v=okTb1Qdp6X0

[^10]: https://www.mongodb.com/developer/videos/everything-you-wanted-to-know-about-rust-unit-testing-and-then-some-more-/

[^11]: https://news.ycombinator.com/item?id=43419701

[^12]: https://rust-lang.github.io/rust-by-example/testing/integration_testing.html

[^13]: https://users.rust-lang.org/t/unit-tests-for-mongo-insert-call/72069

[^14]: https://blog.devgenius.io/how-to-use-mongodb-with-rust-8bd2fa3f6074?gi=b6a482e1e19d

[^15]: https://www.mongodb.com/developer/languages/rust/

[^16]: https://dev.to/hackmamba/build-a-rest-api-with-rust-and-mongodb-rocket-version-ah5

[^17]: https://www.youtube.com/watch?v=9_1hkYVQ1eI

[^18]: https://www.youtube.com/watch?v=C4Kaq6oLvoM

[^19]: https://www.youtube.com/watch?v=HFKumuj4z28

[^20]: https://www.youtube.com/watch?v=EQfGDIyC7ng

[^21]: https://www.youtube.com/watch?v=SzEBpNB_Z1E

[^22]: https://www.youtube.com/watch?v=MiLp0InGOkI

[^23]: https://www.youtube.com/watch?v=fm6oinNergw

