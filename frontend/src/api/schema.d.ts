export interface paths {
    "/documents": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** @description Placeholder: returns an empty response. */
        get: operations["get_documents"];
        put?: never;
        /** @description Placeholder: returns an empty response. */
        post: operations["classify_documents"];
        delete?: never;
        /** @description Placeholder: returns an empty response. */
        options: operations["query_documents"];
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trainers": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** @description Validate label IDs and store an untrained trainer in memory. Does not fit a model. */
        post: operations["create_trainer"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trainers/{trainer_id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** @description Read trainer configuration, lifecycle state, error, or resulting model ID. */
        get: operations["trainer_metadata"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/trainers/{trainer_id}/train": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get?: never;
        put?: never;
        /** @description Train and evaluate once, awaiting completion. Returns a new model ID. Disconnecting does not cancel training. */
        post: operations["train_model"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/models/{model_id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** @description Read completed model metadata and evaluation. Resolve current label names through /labels. */
        get: operations["model_metadata"];
        put?: never;
        post?: never;
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/labels": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        /** @description List all labels ordered by ID. */
        get: operations["list_labels"];
        put?: never;
        /** @description Create a label with a MongoDB-generated ID. Duplicate names are allowed. */
        post: operations["create_label"];
        delete?: never;
        options?: never;
        head?: never;
        patch?: never;
        trace?: never;
    };
    "/labels/{id}": {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        get: operations["get_label"];
        put?: never;
        post?: never;
        /** @description Delete a label. Document/trainer reference checks are not implemented. */
        delete: operations["delete_label"];
        options?: never;
        head?: never;
        patch: operations["rename_label"];
        trace?: never;
    };
}
export type webhooks = Record<string, never>;
export interface components {
    schemas: {
        CreateTrainerRequest: {
            classifier: components["schemas"]["ClassifierType"];
            documents: components["schemas"]["TrainingDocument"][];
            /**
             * @default {
             *       "ratio": 0.800000011920929,
             *       "repetitions": 5,
             *       "sampling": "stratified"
             *     }
             */
            split: components["schemas"]["SplitSettings"];
            /**
             * @default {
             *       "tf_scheme": "raw_count",
             *       "idf_scheme": "base",
             *       "double_normalization_k": 0.5
             *     }
             */
            tf_idf: components["schemas"]["TfIdfSettings"];
            /** @default macro_f1 */
            evaluation_strategy: components["schemas"]["EvaluationStrategy"];
            /**
             * Format: uint64
             * @description Random seed for reproducible splits. Generated and returned when omitted.
             */
            seed?: number | null;
        };
        /**
         * @description Supported model types. Add variants and their training dispatch together.
         * @enum {string}
         */
        ClassifierType: "nearest_centroid";
        TrainingDocument: {
            name: string;
            pages: string[];
            /** @description Hexadecimal MongoDB label ID. Each class needs at least two documents. */
            label_id: string;
        };
        SplitSettings: {
            /**
             * Format: float
             * @description Training fraction, strictly between zero and one.
             * @default 0.800000011920929
             */
            ratio: number;
            /**
             * Format: uint
             * @description Number of train/test splits; must be at least one.
             * @default 5
             */
            repetitions: number;
            /** @default stratified */
            sampling: components["schemas"]["SamplingStrategy"];
        };
        /** @enum {string} */
        SamplingStrategy: "random" | "stratified";
        TfIdfSettings: {
            /** @default raw_count */
            tf_scheme: components["schemas"]["TfWeightScheme"];
            /** @default base */
            idf_scheme: components["schemas"]["IdfWeightScheme"];
            /**
             * Format: double
             * @default 0.5
             */
            double_normalization_k: number;
        };
        /** @enum {string} */
        TfWeightScheme: "binary" | "raw_count" | "normalization" | "double_normalization_half" | "double_normalization_variable";
        /** @enum {string} */
        IdfWeightScheme: "base" | "smooth" | "maximum" | "probabilistic";
        /** @enum {string} */
        EvaluationStrategy: "macro_f1" | "balanced_accuracy" | "matthews_correlation";
        TrainerMetadata: {
            /** Format: uuid */
            trainer_id: string;
            classifier: components["schemas"]["ClassifierType"];
            /** Format: uint64 */
            seed: number;
            label_ids: string[];
            split: components["schemas"]["SplitSettings"];
            tf_idf: components["schemas"]["TfIdfSettings"];
            evaluation_strategy: components["schemas"]["EvaluationStrategy"];
            status: components["schemas"]["TrainerStatus"];
            /** Format: uuid */
            model_id?: string | null;
            error?: string | null;
        };
        /** @enum {string} */
        TrainerStatus: "created" | "training" | "completed" | "failed";
        ErrorResponse: {
            error: string;
        };
        TrainerPath: {
            /** Format: uuid */
            trainer_id: string;
        };
        ModelMetadata: {
            /** Format: uuid */
            model_id: string;
            /** Format: uuid */
            trainer_id: string;
            classifier: components["schemas"]["ClassifierType"];
            /** Format: uint64 */
            seed: number;
            label_ids: string[];
            /** Format: uint */
            feature_count: number;
            split: components["schemas"]["SplitSettings"];
            tf_idf: components["schemas"]["TfIdfSettings"];
            evaluation: components["schemas"]["EvaluationReport"];
        };
        EvaluationReport: {
            strategy: components["schemas"]["EvaluationStrategy"];
            /** Format: uint64 */
            seed: number;
            score_usage: string;
            /** Format: uint */
            selected_split: number;
            splits: components["schemas"]["SplitEvaluation"][];
            train_summary: components["schemas"]["MetricSummary"];
            test_summary: components["schemas"]["MetricSummary"];
            gap_summary: components["schemas"]["MetricSummary"];
        };
        SplitEvaluation: {
            /** Format: uint */
            split_index: number;
            /** Format: uint */
            train_samples: number;
            /** Format: uint */
            test_samples: number;
            train: components["schemas"]["Metrics"];
            test: components["schemas"]["Metrics"];
            gap: components["schemas"]["Metrics"];
        };
        Metrics: {
            /** Format: double */
            macro_f1: number;
            /** Format: double */
            balanced_accuracy: number;
            /** Format: double */
            matthews_correlation: number;
        };
        MetricSummary: {
            mean: components["schemas"]["Metrics"];
            /** @description Population standard deviation across the observed splits. */
            standard_deviation: components["schemas"]["Metrics"];
        };
        ModelPath: {
            /** Format: uuid */
            model_id: string;
        };
        LabelInput: {
            name: string;
        };
        LabelResponse: {
            id: string;
            name: string;
        };
        LabelPath: {
            id: string;
        };
    };
    responses: never;
    parameters: never;
    requestBodies: never;
    headers: never;
    pathItems: never;
}
export type $defs = Record<string, never>;
export interface operations {
    get_documents: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description no content */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    classify_documents: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description no content */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    query_documents: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description no content */
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
        };
    };
    create_trainer: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["CreateTrainerRequest"];
            };
        };
        responses: {
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["TrainerMetadata"];
                };
            };
            400: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            422: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            500: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
        };
    };
    trainer_metadata: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                trainer_id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["TrainerMetadata"];
                };
            };
            /** @description plain text */
            400: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "text/plain; charset=utf-8": unknown;
                };
            };
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
        };
    };
    train_model: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                trainer_id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ModelMetadata"];
                };
            };
            /** @description plain text */
            400: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "text/plain; charset=utf-8": unknown;
                };
            };
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            409: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            422: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            500: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
        };
    };
    model_metadata: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                model_id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ModelMetadata"];
                };
            };
            /** @description plain text */
            400: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "text/plain; charset=utf-8": unknown;
                };
            };
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
        };
    };
    list_labels: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["LabelResponse"][];
                };
            };
            500: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
        };
    };
    create_label: {
        parameters: {
            query?: never;
            header?: never;
            path?: never;
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["LabelInput"];
            };
        };
        responses: {
            201: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["LabelResponse"];
                };
            };
            400: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            422: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            500: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
        };
    };
    get_label: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["LabelResponse"];
                };
            };
            400: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            500: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
        };
    };
    delete_label: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody?: never;
        responses: {
            /** @description no content */
            204: {
                headers: {
                    [name: string]: unknown;
                };
                content?: never;
            };
            400: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            500: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
        };
    };
    rename_label: {
        parameters: {
            query?: never;
            header?: never;
            path: {
                id: string;
            };
            cookie?: never;
        };
        requestBody: {
            content: {
                "application/json": components["schemas"]["LabelInput"];
            };
        };
        responses: {
            200: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["LabelResponse"];
                };
            };
            400: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            404: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            422: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
            500: {
                headers: {
                    [name: string]: unknown;
                };
                content: {
                    "application/json": components["schemas"]["ErrorResponse"];
                };
            };
        };
    };
}
