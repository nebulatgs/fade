use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/gql/schema.graphql",
    query_path = "src/gql/queries/strings/GetOrganizationMeta.graphql",
    response_derives = "Debug"
)]
pub struct GetOrganizationMeta;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/gql/schema.graphql",
    query_path = "src/gql/queries/strings/GetAppMeta.graphql",
    response_derives = "Debug"
)]
pub struct GetAppMeta;
