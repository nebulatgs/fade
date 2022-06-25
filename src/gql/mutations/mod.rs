use graphql_client::GraphQLQuery;

use super::machine_config::MachineConfig;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/gql/schema.graphql",
    query_path = "src/gql/mutations/strings/CreateApp.graphql",
    response_derives = "Debug"
)]
pub struct CreateApp;

type JSON = MachineConfig;
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/gql/schema.graphql",
    query_path = "src/gql/mutations/strings/LaunchMachine.graphql",
    response_derives = "Debug"
)]
pub struct LaunchMachine;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/gql/schema.graphql",
    query_path = "src/gql/mutations/strings/RemoveMachine.graphql",
    response_derives = "Debug"
)]
pub struct RemoveMachine;
