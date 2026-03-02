// SPDX-License-Identifier: PMPL-1.0-or-later
//! GraphQL schema type definitions

use async_graphql::{EmptySubscription, Schema};

use crate::resolvers::{MutationRoot, QueryRoot};

/// Application GraphQL schema
pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;
