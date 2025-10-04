// @generated automatically by Diesel

diesel::table! {
    documents (filepath) {
        filepath -> Text,
        id -> Text,
        title -> Text,
        document_type -> Text,
        created_at -> Double,
        updated_at -> Double,
        archived -> Bool,
        exit_criteria_met -> Bool,
        file_hash -> Text,
        frontmatter_json -> Text,
        content -> Nullable<Text>,
        phase -> Text,
        strategy_id -> Nullable<Text>,
        initiative_id -> Nullable<Text>,
    }
}

diesel::table! {
    document_relationships (child_filepath, parent_filepath) {
        child_id -> Text,
        parent_id -> Text,
        child_filepath -> Text,
        parent_filepath -> Text,
    }
}

diesel::table! {
    document_search (rowid) {
        rowid -> Integer,
        document_filepath -> Text,
        content -> Nullable<Text>,
        title -> Nullable<Text>,
        document_type -> Nullable<Text>,
    }
}

diesel::table! {
    document_tags (document_filepath, tag) {
        document_filepath -> Text,
        tag -> Text,
    }
}

diesel::table! {
    configuration (key) {
        key -> Text,
        value -> Text,
        updated_at -> Double,
    }
}

diesel::joinable!(document_tags -> documents (document_filepath));
diesel::joinable!(document_search -> documents (document_filepath));

diesel::allow_tables_to_appear_in_same_query!(
    documents,
    document_relationships,
    document_search,
    document_tags,
    configuration,
);
