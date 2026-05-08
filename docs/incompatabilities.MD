// TODO:// Incompatible Changes Detected During Build
The following warnings were discovered during the build. These warnings are an
indication that the packages contain code that will become an error in a
future release of Rust. These warnings typically cover changes to close
soundness problems, unintended or undocumented behavior, or critical problems
that cannot be fixed in a backwards-compatible fashion, and are not expected
to be in wide use.

Each warning should contain a link for more information on what the warning
means and how to resolve it.


To solve this problem, you can try the following approaches:


- Some affected dependencies have newer versions available.
You may want to consider updating them to a newer version to see if the issue has been fixed.

sqlx-postgres v0.7.4 has the following newer versions available: 0.8.0, 0.8.1, 0.8.2, 0.8.3, 0.8.5, 0.8.6, 0.9.0-alpha.1

- If the issue is not solved by updating the dependencies, a fix has to be
implemented by those dependencies. You can help with that by notifying the
maintainers of this problem (e.g. by creating a bug report) or by proposing a
fix to the maintainers (e.g. by creating a pull request):

  - sqlx-postgres@0.7.4
  - Repository: https://github.com/launchbadge/sqlx
  - Detailed warning command: `cargo report future-incompatibilities --id 1 --package sqlx-postgres@0.7.4`

- If waiting for an upstream fix is not an option, you can use the `[patch]`
section in `Cargo.toml` to use your own version of the dependency. For more
information, see:
https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html#the-patch-section

The package `sqlx-postgres v0.7.4` currently triggers the following future incompatibility lints:
> warning: this function depends on never type fallback being `()`
>   --> /Users/Terry/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sqlx-postgres-0.7.4/src/connection/executor.rs:23:1
>    |
> 23 | / async fn prepare(
> 24 | |     conn: &mut PgConnection,
> 25 | |     sql: &str,
> 26 | |     parameters: &[PgTypeInfo],
> 27 | |     metadata: Option<Arc<PgStatementMetadata>>,
> 28 | | ) -> Result<(Oid, Arc<PgStatementMetadata>), Error> {
>    | |___________________________________________________^
>    |
>    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in Rust 2024 and in a future release in all editions!
>    = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/never-type-fallback.html>
>    = help: specify the types explicitly
> note: in edition 2024, the requirement `!: sqlx_core::io::Decode<'_>` will fail
>   --> /Users/Terry/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sqlx-postgres-0.7.4/src/connection/executor.rs:68:10
>    |
> 68 |         .recv_expect(MessageFormat::ParseComplete)
>    |          ^^^^^^^^^^^
> help: use `()` annotations to avoid fallback changes
>    |
> 66 |     let _: () = conn
>    |          ++++
> 
> warning: this function depends on never type fallback being `()`
>    --> /Users/Terry/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sqlx-postgres-0.7.4/src/copy.rs:262:5
>     |
> 262 |     pub async fn abort(mut self, msg: impl Into<String>) -> Result<()> {
>     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
>     |
>     = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in Rust 2024 and in a future release in all editions!
>     = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/never-type-fallback.html>
>     = help: specify the types explicitly
> note: in edition 2024, the requirement `!: sqlx_core::io::Decode<'_>` will fail
>    --> /Users/Terry/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sqlx-postgres-0.7.4/src/copy.rs:280:30
>     |
> 280 | ...                   .recv_expect(MessageFormat::ReadyForQuery)
>     |                        ^^^^^^^^^^^
> help: use `()` annotations to avoid fallback changes
>     |
> 280 |                             .recv_expect::<()>(MessageFormat::ReadyForQuery)
>     |                                         ++++++
> 
> warning: this function depends on never type fallback being `()`
>    --> /Users/Terry/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sqlx-postgres-0.7.4/src/copy.rs:294:5
>     |
> 294 |     pub async fn finish(mut self) -> Result<u64> {
>     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
>     |
>     = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in Rust 2024 and in a future release in all editions!
>     = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/never-type-fallback.html>
>     = help: specify the types explicitly
> note: in edition 2024, the requirement `!: sqlx_core::io::Decode<'_>` will fail
>    --> /Users/Terry/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sqlx-postgres-0.7.4/src/copy.rs:314:14
>     |
> 314 |             .recv_expect(MessageFormat::ReadyForQuery)
>     |              ^^^^^^^^^^^
> help: use `()` annotations to avoid fallback changes
>     |
> 314 |             .recv_expect::<()>(MessageFormat::ReadyForQuery)
>     |                         ++++++
> 
> warning: this function depends on never type fallback being `()`
>    --> /Users/Terry/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sqlx-postgres-0.7.4/src/copy.rs:331:1
>     |
> 331 | / async fn pg_begin_copy_out<'c, C: DerefMut<Target = PgConnection> + Send + 'c>(
> 332 | |     mut conn: C,
> 333 | |     statement: &str,
> 334 | | ) -> Result<BoxStream<'c, Result<Bytes>>> {
>     | |_________________________________________^
>     |
>     = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in Rust 2024 and in a future release in all editions!
>     = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/never-type-fallback.html>
>     = help: specify the types explicitly
> note: in edition 2024, the requirement `!: sqlx_core::io::Decode<'_>` will fail
>    --> /Users/Terry/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sqlx-postgres-0.7.4/src/copy.rs:350:33
>     |
> 350 |                     conn.stream.recv_expect(MessageFormat::CommandComplete).await?;
>     |                                 ^^^^^^^^^^^
> help: use `()` annotations to avoid fallback changes
>     |
> 350 ~                     conn.stream.recv_expect::<()>(MessageFormat::CommandComplete).await?;
> 351 ~                     conn.stream.recv_expect::<()>(MessageFormat::ReadyForQuery).await?;
>     |
> 
