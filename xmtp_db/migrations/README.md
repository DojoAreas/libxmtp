# Steps for setting up a diesel migration

### Install the CLI onto your local system (one-time)

```
cargo install diesel_cli --no-default-features --features sqlite
```

### Create your migration SQL

In this example the migration is called `create_key_store`:

```
cd xmtp_db/
diesel migration generate create_key_store
```

Edit the `up.sql` and `down.sql` files created

### Generate application code

```
cd xmtp_db/
cargo run --bin update-schema --features update-schema
```

This updates the generated `schema_gen.rs` file. You can now update the models and queries to reference it in `xmtp_db/src/encrypted_store/`.
