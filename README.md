# maki

## Startup

Provide `MONGODB_URI` and `MONGODB_DATABASE` in the process environment, then run
`cargo run`. The server listens at `127.0.0.1:3000`. Both variables are required;
no credentials or fallback values are supplied. Put authentication options in
your connection URI if your MongoDB deployment requires them.

The shared MongoDB client is initialized at startup without a ping or application
query. Initialization does not verify connectivity. Label operations use the
`labels` collection in the configured database.

For the local database, run `docker compose up -d` (or `mise run db:up`). The
Compose database is bound to localhost and uses authentication with the credentials
configured in `docker-compose.yaml`. In MongoDB Compass, use host `127.0.0.1`,
port `27017`, Username / Password authentication, and authentication database
`admin`. For the app, set `MONGODB_DATABASE` to `maki` and use this URI, replacing
the placeholders with the Compose credentials (percent-encode special characters):

```text
mongodb://USERNAME:PASSWORD@127.0.0.1:27017/?authSource=admin
```

The `mongodb_authenticated_data` volume starts with an empty database so the initial
credentials are created. The previous `mongodb_data` volume is retained but is no
longer mounted. Later credential changes in Compose do not update users in an
existing database. The health check verifies authentication as well as connectivity.

Mise also provides `dev`, `frontend:install`, `frontend:dev`, and
`check` tasks; run `mise tasks` for the full list.

## API documentation

Open `http://127.0.0.1:3000/swagger` for Swagger UI. It loads `/openapi.json`, the
OpenAPI 3.1 specification generated from the same Utoipa-annotated handlers used
by the application. The document endpoints remain placeholders.

Each service keeps business logic in `mod.rs`, route registration in `api/mod.rs`,
HTTP handling in `api/router.rs`, API inputs in `api/request.rs`, and API outputs in
`api/response.rs`. Shared HTTP error conversion lives in `src/api/response.rs`.
Router composition and OpenAPI setup stay in `src/api/mod.rs`. See [AGENTS.md](AGENTS.md).

## Labels

Create labels before creating a trainer. Only a name is accepted; the MongoDB
driver generates the ObjectId. API IDs are 24-character hexadecimal strings.

```sh
curl -s http://127.0.0.1:3000/labels \
  -H 'Content-Type: application/json' \
  -d '{"name":"Fruit"}'
```

The response is `201 Created` with `{ "id": "<label-id>", "name": "Fruit" }`.
Create another label for Vehicle and use the returned IDs in training documents.

| Endpoint | Behavior |
|---|---|
| `POST /labels` | Create from `{ "name": "Fruit" }`; return `201` |
| `GET /labels` | List all labels, sorted by ID |
| `GET /labels/{id}` | Retrieve a label |
| `PATCH /labels/{id}` | Rename using `{ "name": "New name" }` |
| `DELETE /labels/{id}` | Delete the label; return `204` |

Names are trimmed and must not be blank. Duplicate names are allowed. Label IDs
are immutable. Deletion does not check document, trainer, or model references;
an existing model can retain an ID that no longer resolves through `/labels`.

Classifiers store IDs rather than names. Resolve IDs through the label API to
display current names, including changes made after training.

## Create a trainer, then train

`POST /trainers` validates the configuration and verifies that the supplied label
IDs exist. It stores an untrained trainer in memory and returns `201 Created`.
It does not fit TF-IDF or train a classifier. Replace the ID placeholders below
with IDs returned by the label API.

```sh
curl -s http://127.0.0.1:3000/trainers \
  -H 'Content-Type: application/json' \
  -d '{
    "classifier": "nearest_centroid",
    "documents": [
      {"name":"fruit-a", "pages":["apple sweet fruit"], "label_id":"<fruit-label-id>"},
      {"name":"fruit-b", "pages":["apple fresh fruit"], "label_id":"<fruit-label-id>"},
      {"name":"vehicle-a", "pages":["car road vehicle"], "label_id":"<vehicle-label-id>"},
      {"name":"vehicle-b", "pages":["car fast vehicle"], "label_id":"<vehicle-label-id>"}
    ],
    "split": {"ratio":0.8, "repetitions":5, "sampling":"stratified"},
    "tf_idf": {"tf_scheme":"raw_count", "idf_scheme":"base", "double_normalization_k":0.5},
    "evaluation_strategy":"macro_f1",
    "seed":42
  }'
```

The response includes `trainer_id`, `seed`, `label_ids`, configuration,
`status: "created"`, `model_id: null`, and `error: null`.

Start training explicitly using the returned trainer ID:

```sh
curl -s -X POST http://127.0.0.1:3000/trainers/<trainer-id>/train
```

This request waits for fitting and evaluation, then returns `201 Created` with a
new `model_id`, the original `trainer_id`, label IDs, feature count, settings,
seed, and evaluation report. CPU work runs outside the HTTP executor. Once
started, training and result publication continue if the HTTP client disconnects.

| Endpoint | Behavior |
|---|---|
| `POST /trainers` | Create an untrained trainer |
| `GET /trainers/{trainer_id}` | Read configuration, lifecycle state, error, or resulting model ID |
| `POST /trainers/{trainer_id}/train` | Train once and await the resulting model |
| `GET /models/{model_id}` | Read completed model metadata and evaluation |

Trainer states are `created`, `training`, `completed`, and `failed`. There are no
percentage counters or SSE endpoints. Repeated/concurrent training requests for
the same trainer return `409`; create a new trainer after failure. Editing and
retraining are deferred. The former `POST /models` creation endpoint is removed.

`classifier` and `documents` are required. Only `nearest_centroid` is supported.
Each category needs at least two documents. Defaults are an 80% training ratio,
five stratified splits, base/raw-count TF-IDF, and macro-F1. An omitted seed is
generated, returned, and reused when training starts.

Malformed requests/IDs return `400`; invalid settings, unknown training label IDs,
and training validation failures return `422`; missing resources return `404`.
Database or unexpected worker failures return `500`. Failed training stores an
error on trainer metadata and publishes no model.

## Evaluation and selection

The trainer computes `macro_f1`, `balanced_accuracy`, and `matthews_correlation`
for every split's training and test predictions. Reports contain absolute gaps,
actual partition sizes, means, and population standard deviations across splits.

Selection chooses the highest test score for the configured metric, then the
smallest train/test gap on an exact score tie, then the lowest split index.
These are **held-out selection scores**, not an independent final-test estimate.
The winner retains its matching fitted TF-IDF, without retraining on test data.

Stratified sampling shuffles each class independently and assigns
`floor(class_count * ratio)` documents to training, clamped to leave at least one
sample of every class in each partition. This can change the effective ratio for
small classes. `sampling: "random"` uses unstratified partitioning; classes may be
absent from a partition. Evaluated training requires a ratio strictly between zero
and one and nonempty training/test partitions.

TF-IDF is fitted only on training documents. Test documents are transformed using
that fitted state and never passed to classifier training. A fixed seed and
identical input/settings reproduce splits within the same implementation/version.

TF schemes: `binary`, `raw_count`, `normalization`, `double_normalization_half`,
`double_normalization_variable`. IDF schemes: `base`, `smooth`, `maximum`,
`probabilistic`. The double-normalization constant must be within `[0, 1]`.

## Storage and Rust API

Only labels persist in MongoDB. Trainers and selected models remain in memory and
are lost on restart. No document/trainer collections, model serialization,
model-download endpoint, or HTTP prediction endpoint are included.

`ModelTrainer<T, C>::default()` creates an untrained configuration. Set documents,
split/sampling settings, seed, evaluation strategy, TF-IDF builder, and the model
factory through its setters. Call `.train()` explicitly to get an owned
`TrainingOutcome<C>` containing the selected classifier, its matching fitted
TF-IDF, and the evaluation report. There is no separate trainer builder or
`.build()` step. The Rust default remains one split; the HTTP default is five.

The factory receives only training documents and their feature rows in matching
order. A missing factory returns `Missing model generating function!`. The factory
must be `Send` so the service can run CPU work off the HTTP executor.

`LabeledDocument::label_id()` returns a MongoDB `ObjectId`.
`Classifier::train(features, label_ids)` fits one classifier;
`Classifier::predict(features)` returns IDs in input-row order;
`Classifier::labels()` returns distinct learned IDs sorted by ID. Numeric class
labels are no longer accepted. Feature values and evaluation scores remain
floating-point values.

`ClassificationService::model(model_id)` returns the selected classifier/TF-IDF
pair for a completed model. Transform new documents with that model's TF-IDF
before predicting; resolve predicted IDs through `LabelService` for names.

Run `cargo test`, `cargo clippy --all-targets`, and `cargo fmt --check` to verify.
The unit and direct-service tests do not connect to MongoDB. Label CRUD has no
automated tests in this PoC.
