// SPDX-License-Identifier: MPL-2.0
//! GraphQL schema type definitions

use async_graphql::{EmptySubscription, Schema};

use crate::resolvers::{MutationRoot, QueryRoot};

/// Application GraphQL schema
pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;
