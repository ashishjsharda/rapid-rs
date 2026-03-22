//! GraphQL schema builder

use async_graphql::{ObjectType, SubscriptionType, Schema, SchemaBuilder as AsyncSchemaBuilder};

/// Convenience wrapper around async-graphql's SchemaBuilder
pub struct SchemaBuilder<Q, M, S>
where
    Q: ObjectType + 'static,
    M: ObjectType + 'static,
    S: SubscriptionType + 'static,
{
    inner: AsyncSchemaBuilder<Q, M, S>,
}

impl<Q, M, S> SchemaBuilder<Q, M, S>
where
    Q: ObjectType + 'static,
    M: ObjectType + 'static,
    S: SubscriptionType + 'static,
{
    /// Create a new SchemaBuilder
    pub fn new(query: Q, mutation: M, subscription: S) -> Self {
        Self {
            inner: Schema::build(query, mutation, subscription),
        }
    }

    /// Set maximum query depth
    pub fn max_depth(self, depth: usize) -> Self {
        Self {
            inner: self.inner.limit_depth(depth),
        }
    }

    /// Set maximum query complexity
    pub fn max_complexity(self, complexity: usize) -> Self {
        Self {
            inner: self.inner.limit_complexity(complexity),
        }
    }

    /// Add data to the schema context
    pub fn data<T: Send + Sync + 'static>(self, value: T) -> Self {
        Self {
            inner: self.inner.data(value),
        }
    }

    /// Build the schema
    pub fn finish(self) -> Schema<Q, M, S> {
        self.inner.finish()
    }
}
